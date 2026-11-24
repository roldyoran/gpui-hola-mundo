//! Vista del chat: panel centrado sobre la foto de fondo.
//!
//! Maqueta 100 % visual: los mensajes salen de `ChatState::seed()` y ningun
//! control tiene handler. La composicion replica el patron de pantalla de
//! gentle-shell (`ui::screen`): fondo atenuado + panel centrado con cabecera.

use super::state::{ChatState, Message, Role};
use crate::shared::ui;
use gpui_kit::component::label::Label;
use gpui_kit::*;

/// Vista raiz del chat.
pub struct ChatView {
    state: ChatState,
}

impl ChatView {
    /// Crea la vista con los mensajes de ejemplo.
    pub fn new() -> Self {
        Self {
            state: ChatState::seed(),
        }
    }

    /// Icono Tabler de 16 px en el color dado (`svg` pinta con text color).
    fn ticon(path: &'static str, color: Rgba) -> impl IntoElement {
        svg().path(path).w(px(16.)).h(px(16.)).text_color(color)
    }

    /// Cabecera del panel: marca, titulo y acciones (visuales).
    fn header(&self) -> Div {
        ui::screen_header()
            .child(
                div()
                    .w(px(32.))
                    .h(px(32.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_lg()
                    .bg(ui::accent())
                    .child(Self::ticon("icons/send.svg", rgb(0xffffff))),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .child(Label::new("Chat").font_semibold().text_color(ui::fg()))
                    .child(
                        Label::new("Maqueta visual · GPUI + Tabler")
                            .text_sm()
                            .text_color(ui::muted()),
                    ),
            )
            .child(Self::tool_button("icons/plus.svg"))
            .child(Self::tool_button("icons/settings.svg"))
    }

    /// Boton cuadrado de la cabecera: marco + icono Tabler.
    fn tool_button(path: &'static str) -> Div {
        div()
            .w(px(32.))
            .h(px(32.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_lg()
            .border_1()
            .border_color(ui::border())
            .bg(ui::surface())
            .child(Self::ticon(path, ui::muted()))
    }

    /// Lista de mensajes de ejemplo.
    fn transcript(&self) -> Div {
        let mut rows = div().flex().flex_col().gap_3();
        for message in &self.state.messages {
            rows = rows.child(self.message_row(message));
        }
        rows.p(px(20.))
    }

    /// Fila segun quien habla.
    fn message_row(&self, message: &Message) -> Div {
        match message.role {
            Role::Assistant => self.assistant_row(&message.text),
            Role::User => self.user_row(&message.text),
        }
    }

    /// Fila del asistente: avatar + burbuja a la izquierda.
    fn assistant_row(&self, text: &str) -> Div {
        div()
            .flex()
            .flex_row()
            .gap_3()
            .child(
                div()
                    .w(px(28.))
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_lg()
                    .bg(ui::accent())
                    .child(
                        Label::new("G")
                            .font_semibold()
                            .text_sm()
                            .text_color(rgb(0xffffff)),
                    ),
            )
            .child(
                div()
                    .rounded_xl()
                    .bg(ui::surface())
                    .border_1()
                    .border_color(ui::border())
                    .p(px(12.))
                    .max_w(px(560.))
                    .child(
                        Label::new(text.to_string())
                            .font_family(ui::mono())
                            .text_sm()
                            .text_color(ui::fg()),
                    ),
            )
    }

    /// Burbuja del usuario alineada a la derecha.
    fn user_row(&self, text: &str) -> Div {
        div().flex().flex_row().justify_end().child(
            div()
                .rounded_xl()
                .bg(ui::raised())
                .p(px(12.))
                .max_w(px(560.))
                .child(
                    Label::new(text.to_string())
                        .font_family(ui::mono())
                        .text_sm()
                        .text_color(ui::fg()),
                ),
        )
    }

    /// Composer visual: chips de estado + campo con placeholder y enviar.
    fn composer(&self) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p(px(16.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .child(ui::chip("maqueta visual", ui::ChipKind::Neutral))
                    .child(ui::chip("sin backend", ui::ChipKind::Info)),
            )
            .child(
                ui::field()
                    .child(Self::ticon("icons/paperclip.svg", ui::muted()))
                    .child(
                        Label::new("Escríbeme algo…")
                            .font_family(ui::mono())
                            .text_sm()
                            .text_color(ui::faint()),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .w(px(28.))
                            .h(px(28.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_lg()
                            .bg(ui::accent())
                            .child(Self::ticon("icons/send.svg", rgb(0xffffff))),
                    ),
            )
    }
}

impl Render for ChatView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .size_full()
            .bg(rgb(0x0b0b0e))
            // Foto de fondo a pantalla completa.
            .child(
                img("images/slava-jamm.jpg")
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h_full()
                    .object_fit(ObjectFit::Cover),
            )
            // Velo para legibilidad.
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h_full()
                    .bg(rgba(0x000000a6)),
            )
            // Panel de chat centrado.
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .w_full()
                            .max_w(px(760.))
                            .max_h_full()
                            .flex()
                            .flex_col()
                            .rounded_2xl()
                            .border_1()
                            .border_color(ui::border_strong())
                            .bg(ui::panel())
                            .shadow_2xl()
                            .overflow_hidden()
                            .child(self.header())
                            .child(self.transcript())
                            .child(self.composer()),
                    ),
            )
    }
}
