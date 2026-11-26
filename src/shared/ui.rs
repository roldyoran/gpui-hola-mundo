//! Tokens y piezas visuales compartidas.
//!
//! La paleta replica las variables de **shadcn/ui en modo oscuro** y la
//! composición sigue a `gentle-shell-gpui`: cabecera fija, cuerpo con scroll,
//! secciones con título y descripción, cards y pie con la acción principal.
//! Ningún feature inventa colores, radios ni alturas: todo sale de aquí.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

// ── Tokens (shadcn/ui · dark) ─────────────────────────────────────────────

/// `--background`: fondo de la ventana.
pub fn window_bg() -> Rgba {
    rgba(0x09090bff)
}

/// `--card`: superficie de las tarjetas, apenas por encima del fondo.
pub fn card_bg() -> Rgba {
    rgba(0x0c0c0eff)
}

/// `--secondary` / `--muted`: pozos, filas y fondos de control.
pub fn muted_bg() -> Rgba {
    rgba(0x18181bff)
}

/// `--accent`: hover de filas y botones fantasma.
pub fn hover_bg() -> Rgba {
    rgba(0x27272aff)
}

/// Fondo de un control pulsado.
pub fn active_bg() -> Rgba {
    rgba(0x3f3f46ff)
}

/// `--border` / `--input`.
pub fn border() -> Rgba {
    rgba(0x27272aff)
}

/// Separadores internos, un paso por debajo del borde.
pub fn border_subtle() -> Rgba {
    rgba(0x1f1f23ff)
}

/// `--foreground`.
pub fn fg() -> Rgba {
    rgb(0xfafafa)
}

/// `--muted-foreground`.
pub fn muted() -> Rgba {
    rgb(0xa1a1aa)
}

/// Texto terciario y placeholders.
pub fn faint() -> Rgba {
    rgb(0x71717a)
}

/// `--primary`.
pub fn primary() -> Rgba {
    rgb(0xfafafa)
}

/// `--primary-foreground`.
pub fn primary_fg() -> Rgba {
    rgb(0x18181b)
}

pub fn primary_hover() -> Rgba {
    rgb(0xe4e4e7)
}

pub fn primary_active() -> Rgba {
    rgb(0xd4d4d8)
}

/// Estado activo del micrófono.
pub fn success() -> Rgba {
    rgb(0x34d399)
}

/// Umbral y avisos.
pub fn warning() -> Rgba {
    rgb(0xfbbf24)
}

/// Errores de lectura, nunca para decorar.
pub fn danger() -> Rgba {
    rgb(0xf87171)
}

/// `--destructive`.
pub fn destructive() -> Rgba {
    rgb(0x7f1d1d)
}

pub fn destructive_hover() -> Rgba {
    rgb(0x991b1b)
}

pub fn destructive_active() -> Rgba {
    rgb(0x450a0a)
}

// ── Piezas ───────────────────────────────────────────────────────────────

/// Sección: título, descripción y la card que la acompaña.
pub fn section(title: impl Into<SharedString>, description: impl Into<SharedString>) -> Div {
    div().flex().flex_col().gap_3().child(
        div()
            .flex()
            .flex_col()
            .gap_1()
            .px(px(2.))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(fg())
                    .child(title.into()),
            )
            .child(hint(description)),
    )
}

/// Tarjeta base, sin padding: las filas ponen el suyo.
pub fn card() -> Div {
    div()
        .flex()
        .flex_col()
        .rounded_lg()
        .bg(card_bg())
        .border_1()
        .border_color(border())
        .overflow_hidden()
}

/// Cuerpo de card con aire uniforme.
pub fn card_body() -> Div {
    div().flex().flex_col().gap_4().p(px(16.))
}

/// Fila de lista o ajuste.
pub fn row() -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_3()
        .h(px(52.))
        .px(px(12.))
        .rounded_md()
}

/// Cabecera de la ventana: marca, título, subtítulo y acción.
pub fn screen_header() -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_4()
        .flex_shrink_0()
        .h(px(64.))
        .px(px(24.))
        .border_b_1()
        .border_color(border())
}

pub fn divider() -> Div {
    div().h(px(1.)).w_full().bg(border_subtle())
}

/// Texto principal de un control.
pub fn label(text: impl Into<SharedString>) -> Div {
    div().text_sm().text_color(fg()).child(text.into())
}

/// Texto de apoyo.
pub fn hint(text: impl Into<SharedString>) -> Div {
    div().text_xs().text_color(faint()).child(text.into())
}

/// Micro etiqueta en mayúsculas.
pub fn section_label(text: impl Into<SharedString>) -> Div {
    div()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(faint())
        .child(text.into().to_uppercase())
}

/// Base compartida por las variantes de botón, sin estados.
fn button_base(text: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .h(px(32.))
        .px(px(12.))
        .rounded_md()
        .cursor_pointer()
        .border_1()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .child(text.into())
}

/// Botón secundario: fondo oscuro y borde.
pub fn button(text: impl Into<SharedString>) -> Div {
    button_base(text)
        .bg(card_bg())
        .border_color(border())
        .text_color(fg())
        .hover(|style| style.bg(hover_bg()).border_color(hover_bg()))
        .active(|style| style.bg(active_bg()).border_color(active_bg()))
}

/// Botón principal: claro sobre oscuro, como el `default` de shadcn.
pub fn primary_button(text: impl Into<SharedString>) -> Div {
    button_base(text)
        .bg(primary())
        .border_color(primary())
        .text_color(primary_fg())
        .hover(|style| style.bg(primary_hover()).border_color(primary_hover()))
        .active(|style| style.bg(primary_active()).border_color(primary_active()))
}

/// Botón fantasma para acciones discretas.
pub fn ghost_button(text: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .h(px(32.))
        .px(px(10.))
        .rounded_md()
        .cursor_pointer()
        .text_xs()
        .text_color(muted())
        .hover(|style| style.bg(hover_bg()).text_color(fg()))
        .active(|style| style.bg(active_bg()).text_color(fg()))
        .child(text.into())
}

/// Botón de icono cuadrado (cerrar, pasos −/+).
pub fn icon_button(icon: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(32.))
        .h(px(32.))
        .rounded_md()
        .cursor_pointer()
        .text_lg()
        .text_color(muted())
        .hover(|style| style.bg(hover_bg()).text_color(fg()))
        .active(|style| style.bg(active_bg()).text_color(fg()))
        .child(icon.into())
}

/// Botón destructivo sólido.
pub fn destructive_button(text: impl Into<SharedString>) -> Div {
    button_base(text)
        .bg(destructive())
        .border_color(destructive())
        .text_color(fg())
        .hover(|style| {
            style
                .bg(destructive_hover())
                .border_color(destructive_hover())
        })
        .active(|style| {
            style
                .bg(destructive_active())
                .border_color(destructive_active())
        })
}

/// Switch de 36×20 con knob de 16.
pub fn switch(on: bool) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w(px(36.))
        .h(px(20.))
        .p(px(2.))
        .rounded_full()
        .cursor_pointer()
        .bg(if on { primary() } else { muted_bg() })
        .border_1()
        .border_color(if on { primary() } else { border() })
        .hover(|style| {
            if on {
                style.bg(primary_hover()).border_color(primary_hover())
            } else {
                style.bg(hover_bg()).border_color(active_bg())
            }
        })
        .active(|style| {
            if on {
                style.bg(primary_active()).border_color(primary_active())
            } else {
                style.bg(active_bg())
            }
        })
        .when(on, |element| element.child(div().flex_1()))
        .child(div().w(px(16.)).h(px(16.)).rounded_full().bg(if on {
            primary_fg()
        } else {
            muted()
        }))
}

/// Fila seleccionable de una lista (micrófonos).
pub fn list_row(is_selected: bool) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_3()
        .h(px(40.))
        .px(px(10.))
        .rounded_md()
        .cursor_pointer()
        .bg(if is_selected {
            muted_bg()
        } else {
            rgba(0x00000000)
        })
        .when(!is_selected, |element| {
            element.hover(|style| style.bg(muted_bg()))
        })
}

/// Radio de 16 px: borde y punto interior cuando está elegido.
pub fn radio(is_selected: bool) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(16.))
        .h(px(16.))
        .rounded_full()
        .border_1()
        .border_color(if is_selected { primary() } else { border() })
        .when(is_selected, |element| {
            element.child(div().w(px(8.)).h(px(8.)).rounded_full().bg(primary()))
        })
}

/// Chip de estado: color al 14 % de fondo y texto del mismo tono.
pub fn chip(text: impl Into<SharedString>, color: Rgba) -> Div {
    div()
        .flex()
        .items_center()
        .h(px(22.))
        .px(px(8.))
        .rounded_full()
        .bg(color.opacity(0.14))
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(color)
        .child(text.into())
}

/// Valor numérico compacto.
pub fn value_badge(text: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .h(px(32.))
        .px(px(10.))
        .rounded_md()
        .bg(muted_bg())
        .border_1()
        .border_color(border())
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(fg())
        .child(text.into())
}

/// Contenedor del control segmentado.
pub fn segmented() -> Div {
    div()
        .flex()
        .flex_row()
        .gap(px(2.))
        .p(px(2.))
        .rounded_md()
        .bg(muted_bg())
        .border_1()
        .border_color(border())
}

/// Segmento de un control segmentado.
pub fn segment(text: impl Into<SharedString>, is_selected: bool) -> Div {
    div()
        .flex_1()
        .flex()
        .items_center()
        .justify_center()
        .h(px(28.))
        .rounded_md()
        .cursor_pointer()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .bg(if is_selected {
            card_bg()
        } else {
            rgba(0x00000000)
        })
        .text_color(if is_selected { fg() } else { muted() })
        .border_1()
        .border_color(if is_selected {
            border()
        } else {
            rgba(0x00000000)
        })
        .when(!is_selected, |element| {
            element.hover(|style| style.bg(hover_bg()).text_color(fg()))
        })
        .child(text.into())
}

/// Medidor horizontal con la marca del umbral.
///
/// `fill` y `threshold` van en 0..=1; `speaking` cambia el color del relleno.
pub fn meter(fill: f32, threshold: f32, speaking: bool) -> Div {
    div()
        .relative()
        .w_full()
        .h(px(12.))
        .rounded_full()
        .bg(muted_bg())
        .overflow_hidden()
        .child(
            div()
                .absolute()
                .left(px(0.))
                .top(px(0.))
                .bottom(px(0.))
                .w(relative(fill.clamp(0., 1.)))
                .rounded_full()
                .bg(if speaking { success() } else { primary() }),
        )
        .child(
            div()
                .absolute()
                .left(relative(threshold.clamp(0., 1.)))
                .top(px(0.))
                .bottom(px(0.))
                .w(px(2.))
                .bg(warning()),
        )
}

/// Punto de estado.
pub fn status_dot(color: Rgba) -> Div {
    div().w(px(6.)).h(px(6.)).rounded_full().bg(color)
}

/// Bloque de aviso descartable.
pub fn notice(message: impl Into<SharedString>, color: Rgba) -> Div {
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap_3()
        .p(px(12.))
        .bg(color.opacity(0.08))
        .border_t_1()
        .border_color(color.opacity(0.22))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_xs()
                .text_color(color)
                .child(message.into()),
        )
}
