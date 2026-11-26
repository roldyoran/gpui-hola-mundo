//! Ventana del avatar: sin fondo, sin bordes, arrastrable.
//!
//! La raíz no pinta nada: el `clear` transparente de GPUI deja ver el
//! escritorio y solo se dibuja el PNG del estado actual. La textura se carga
//! una vez y solo se recarga cuando cambian los archivos.

use std::sync::Arc;

use gpui_kit::*;

use crate::app::AppState;
use crate::features::avatar::decode_png;

pub struct AvatarView {
    state: Entity<AppState>,
    generation: u64,
    closed: Option<Arc<Image>>,
    open: Option<Arc<Image>>,
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
                    // Bloquea hasta que el usuario suelta el botón; después se
                    // guarda la posición real que dejó el arrastre nativo.
                    crate::win32::start_drag(window);
                    let bounds = window.bounds();
                    let (x, y, width, height) = (
                        f32::from(bounds.origin.x),
                        f32::from(bounds.origin.y),
                        f32::from(bounds.size.width),
                        f32::from(bounds.size.height),
                    );
                    this.state
                        .update(cx, |state, _| state.set_bounds(x, y, width, height));
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, _: &MouseDownEvent, _window, cx| {
                    let _ = crate::window::open_settings(this.state.clone(), cx);
                }),
            );

        match image {
            Some(image) => root.child(img(image).w_full().h_full().object_fit(ObjectFit::Contain)),
            None => root,
        }
    }
}
