//! Ventana del avatar: sin fondo, sin bordes, arrastrable.
//!
//! La raíz no pinta nada: el `clear` transparente de GPUI deja ver el
//! escritorio y solo se dibuja el PNG del estado actual. La textura se carga
//! una vez y solo se recarga cuando cambian los archivos.
//!
//! El arrastre es manual: el handler solo guarda el punto de partida y una
//! tarea mueve la ventana con `SetWindowPos`. Nunca se abre el bucle modal de
//! `WM_NCLBUTTONDOWN`, que era lo que hacía caer la app al alternar clics.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::*;

use crate::app::AppState;
use crate::features::avatar::decode_png;

/// Cadencia del arrastre manual; suficiente para seguir al cursor.
const DRAG_FRAME: Duration = Duration::from_millis(16);

pub struct AvatarView {
    state: Entity<AppState>,
    generation: u64,
    closed: Option<Arc<Image>>,
    open: Option<Arc<Image>>,
    /// Hay un arrastre en curso; el clic derecho se ignora mientras tanto.
    dragging: bool,
    #[allow(dead_code)]
    drag_task: Option<Task<()>>,
    #[allow(dead_code)]
    observed: Subscription,
}

impl AvatarView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let observed = cx.observe(&state, |this, state, cx| {
            if state.read(cx).avatar_generation != this.generation {
                this.reload(cx);
            }
            cx.notify();
        });

        let mut view = Self {
            state,
            generation: 0,
            closed: None,
            open: None,
            dragging: false,
            drag_task: None,
            observed,
        };
        view.reload(cx);
        view
    }

    /// Recarga ambos PNG desde el directorio de datos.
    fn reload(&mut self, cx: &mut Context<Self>) {
        let (paths, closed, open, generation) = {
            let state = self.state.read(cx);
            (
                state.paths.clone(),
                state.settings.avatar.closed.clone(),
                state.settings.avatar.open.clone(),
                state.avatar_generation,
            )
        };

        self.generation = generation;
        self.closed = closed
            .and_then(|relative| paths.avatar_bytes(&relative))
            .and_then(|bytes| decode_png(&bytes));
        self.open = open
            .and_then(|relative| paths.avatar_bytes(&relative))
            .and_then(|bytes| decode_png(&bytes));
    }

    /// Arranca un arrastre manual y espera a que se suelte el botón.
    fn begin_drag(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.dragging {
            return;
        }
        let bounds = window.bounds();
        let (x, y, width, height) = (
            f32::from(bounds.origin.x),
            f32::from(bounds.origin.y),
            f32::from(bounds.size.width),
            f32::from(bounds.size.height),
        );
        crate::win32::begin_drag(window, window.scale_factor(), x, y, width, height);
        self.dragging = true;
        self.drag_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(DRAG_FRAME).await;
                match crate::win32::drag_step() {
                    Some(crate::win32::DragStep::Active) => {}
                    Some(crate::win32::DragStep::Finished {
                        x,
                        y,
                        width,
                        height,
                    }) => {
                        let _ = this.update(cx, |view, cx| {
                            view.dragging = false;
                            view.state
                                .update(cx, |state, _| state.set_bounds(x, y, width, height));
                        });
                        break;
                    }
                    None => {
                        let _ = this.update(cx, |view, _| view.dragging = false);
                        break;
                    }
                }
            }
        }));
    }
}

impl Render for AvatarView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mouth_open = self.state.read(cx).mouth_open;
        let image = if mouth_open {
            self.open.clone()
        } else {
            self.closed.clone()
        };

        let root = div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _: &MouseDownEvent, window, cx| {
                    this.begin_drag(window, cx);
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, _: &MouseDownEvent, _window, cx| {
                    // Un clic derecho durante el arrastre no debe abrir nada:
                    // el botón izquierdo sigue capturado y el arrastre manda.
                    if this.dragging {
                        return;
                    }
                    let _ = crate::window::open_settings(this.state.clone(), cx);
                }),
            );

        match image {
            Some(image) => root.child(img(image).w_full().h_full().object_fit(ObjectFit::Contain)),
            None => root,
        }
    }
}
