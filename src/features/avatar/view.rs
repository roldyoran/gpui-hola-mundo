//! Ventana del avatar: sin fondo, sin bordes, arrastrable.
//!
//! La raíz no pinta nada: el `clear` transparente de GPUI deja ver el
//! escritorio y solo se dibuja el PNG del estado actual. Las texturas viven en
//! `AppState` y las comparte con la previsualización: se decodifican una vez.
//!
//! El arrastre es manual: el handler solo guarda el punto de partida y una
//! tarea mueve la ventana con `SetWindowPos`. Nunca se abre el bucle modal de
//! `WM_NCLBUTTONDOWN`, que era lo que hacía caer la app al alternar clics.

use std::time::Duration;

use gpui_kit::*;

use crate::app::AppState;
use crate::win32::ResizeEdge;

/// Cadencia del arrastre manual; suficiente para seguir al cursor.
const DRAG_FRAME: Duration = Duration::from_millis(16);

pub struct AvatarView {
    state: Entity<AppState>,
    /// Hay un arrastre en curso; el clic derecho se ignora mientras tanto.
    dragging: bool,
    /// Hay un redimensionado en curso; no se permite arrastrar a la vez.
    resizing: bool,
    #[allow(dead_code)]
    drag_task: Option<Task<()>>,
    #[allow(dead_code)]
    resize_task: Option<Task<()>>,
    #[allow(dead_code)]
    observed: Subscription,
}

impl AvatarView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let observed = cx.observe(&state, |_, _, cx| cx.notify());
        Self {
            state,
            dragging: false,
            resizing: false,
            drag_task: None,
            resize_task: None,
            observed,
        }
    }

    /// Arranca un arrastre manual y espera a que se suelte el botón.
    fn begin_drag(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.dragging || self.resizing || self.state.read(cx).avatar_maximized {
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

    /// Arranca un redimensionado desde un borde o esquina del overlay.
    fn begin_resize(&mut self, edge: ResizeEdge, window: &Window, cx: &mut Context<Self>) {
        if self.dragging || self.resizing || self.state.read(cx).avatar_maximized {
            return;
        }
        let bounds = window.bounds();
        let (x, y, width, height) = (
            f32::from(bounds.origin.x),
            f32::from(bounds.origin.y),
            f32::from(bounds.size.width),
            f32::from(bounds.size.height),
        );
        crate::win32::begin_resize(window, edge, window.scale_factor(), x, y, width, height);
        self.resizing = true;
        self.resize_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(DRAG_FRAME).await;
                match crate::win32::resize_step() {
                    Some(crate::win32::DragStep::Active) => {}
                    Some(crate::win32::DragStep::Finished {
                        x,
                        y,
                        width,
                        height,
                    }) => {
                        let _ = this.update(cx, |view, cx| {
                            view.resizing = false;
                            view.state
                                .update(cx, |state, _| state.set_bounds(x, y, width, height));
                        });
                        break;
                    }
                    None => {
                        let _ = this.update(cx, |view, _| view.resizing = false);
                        break;
                    }
                }
            }
        }));
    }

    /// Zona invisible que inicia el redimensionado desde un borde.
    fn resize_zone(edge: ResizeEdge, zone: Div, cx: &mut Context<Self>) -> Div {
        zone.on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _: &MouseDownEvent, window, cx| {
                this.begin_resize(edge, window, cx);
            }),
        )
    }
}

impl Render for AvatarView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let image = {
            let state = self.state.read(cx);
            if state.mouth_open {
                state.avatar_open.clone()
            } else {
                state.avatar_closed.clone()
            }
        };

        let mut root = div()
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
                    // Un clic derecho durante el arrastre o el redimensionado
                    // no debe abrir nada: el botón izquierdo manda.
                    if this.dragging || this.resizing {
                        return;
                    }
                    let _ = crate::window::open_settings(this.state.clone(), cx);
                }),
            );

        if let Some(image) = image {
            root = root.child(img(image).w_full().h_full().object_fit(ObjectFit::Contain));
        }

        // Ocho zonas invisibles de 8-12 px. Van después del PNG para quedar
        // encima y no pintan nada: solo cambian el cursor y capturan el clic.
        root = root
            .child(Self::resize_zone(
                ResizeEdge::Left,
                div()
                    .absolute()
                    .left(px(0.))
                    .top(px(10.))
                    .bottom(px(10.))
                    .w(px(8.))
                    .cursor_ew_resize(),
                cx,
            ))
            .child(Self::resize_zone(
                ResizeEdge::Right,
                div()
                    .absolute()
                    .right(px(0.))
                    .top(px(10.))
                    .bottom(px(10.))
                    .w(px(8.))
                    .cursor_ew_resize(),
                cx,
            ))
            .child(Self::resize_zone(
                ResizeEdge::Top,
                div()
                    .absolute()
                    .top(px(0.))
                    .left(px(10.))
                    .right(px(10.))
                    .h(px(8.))
                    .cursor_ns_resize(),
                cx,
            ))
            .child(Self::resize_zone(
                ResizeEdge::Bottom,
                div()
                    .absolute()
                    .bottom(px(0.))
                    .left(px(10.))
                    .right(px(10.))
                    .h(px(8.))
                    .cursor_ns_resize(),
                cx,
            ))
            .child(Self::resize_zone(
                ResizeEdge::TopLeft,
                div()
                    .absolute()
                    .top(px(0.))
                    .left(px(0.))
                    .w(px(12.))
                    .h(px(12.))
                    .cursor_nwse_resize(),
                cx,
            ))
            .child(Self::resize_zone(
                ResizeEdge::TopRight,
                div()
                    .absolute()
                    .top(px(0.))
                    .right(px(0.))
                    .w(px(12.))
                    .h(px(12.))
                    .cursor_nesw_resize(),
                cx,
            ))
            .child(Self::resize_zone(
                ResizeEdge::BottomLeft,
                div()
                    .absolute()
                    .bottom(px(0.))
                    .left(px(0.))
                    .w(px(12.))
                    .h(px(12.))
                    .cursor_nesw_resize(),
                cx,
            ))
            .child(Self::resize_zone(
                ResizeEdge::BottomRight,
                div()
                    .absolute()
                    .bottom(px(0.))
                    .right(px(0.))
                    .w(px(12.))
                    .h(px(12.))
                    .cursor_nwse_resize(),
                cx,
            ));

        root
    }
}
