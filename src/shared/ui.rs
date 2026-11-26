//! Tokens y piezas visuales compartidas.
//!
//! Regla de la casa (heredada de `gentle-shell-gpui`): ningún feature inventa
//! colores, radios ni alturas; todo sale de este módulo. Sin dependencias de
//! producto: aquí no se sabe qué es un avatar ni un micrófono.

use gpui_kit::*;

// ── Tokens ───────────────────────────────────────────────────────────────

/// Fondo de la ventana de ajustes.
pub fn panel() -> Rgba {
    rgba(0x121216ff)
}

/// Tarjetas y controles.
pub fn surface() -> Rgba {
    rgba(0x1e1e28ff)
}

/// Huecos oscuros: listas, medidores, fondos de botón.
pub fn well() -> Rgba {
    rgba(0x0e0e13ff)
}

/// Bordes sutiles.
pub fn border() -> Rgba {
    rgba(0xffffff1f)
}

/// Texto principal.
pub fn fg() -> Rgba {
    rgb(0xf2f0eb)
}

/// Texto secundario.
pub fn muted() -> Rgba {
    rgb(0xc9c4bb)
}

/// Texto tenue (pistas, placeholders).
pub fn faint() -> Rgba {
    rgb(0x8a857c)
}

/// Acento de marca: selección, medidor y foco.
pub fn accent() -> Rgba {
    rgb(0xe85d9e)
}

pub fn success() -> Rgba {
    rgb(0x7ee2a8)
}

pub fn warning() -> Rgba {
    rgb(0xf5c97a)
}

pub fn danger() -> Rgba {
    rgb(0xf58a8a)
}

/// Fondo de una fila seleccionada.
pub fn selected() -> Rgba {
    rgba(0xe85d9e26)
}

/// Fondo de botón neutro.
pub fn button_bg() -> Rgba {
    rgba(0x2a2a38ff)
}

/// Fondo de botón destructivo.
pub fn danger_bg() -> Rgba {
    rgba(0x3a1f26ff)
}

// ── Piezas ───────────────────────────────────────────────────────────────

/// La tarjeta estándar: sección de ajustes.
pub fn card() -> Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .p(px(14.))
        .rounded_xl()
        .bg(surface())
        .border_1()
        .border_color(border())
}

/// Cabecera de la ventana: título a la izquierda, acciones a la derecha.
pub fn screen_header() -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .px(px(20.))
        .py(px(14.))
        .border_b_1()
        .border_color(border())
}

/// Micro encabezado en mayúsculas para secciones.
pub fn section_label(text: impl Into<SharedString>) -> Div {
    div()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(faint())
        .child(text.into().to_uppercase())
}

/// Texto principal de un control.
pub fn label(text: impl Into<SharedString>) -> Div {
    div().text_sm().text_color(fg()).child(text.into())
}

/// Texto de apoyo.
pub fn hint(text: impl Into<SharedString>) -> Div {
    div().text_xs().text_color(faint()).child(text.into())
}

/// Botón neutro. El handler lo pone quien lo usa.
pub fn button(text: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .h(px(28.))
        .px(px(12.))
        .rounded_lg()
        .bg(button_bg())
        .border_1()
        .border_color(border())
        .text_sm()
        .text_color(fg())
        .child(text.into())
}

/// Botón de salida, con tinte rojo.
pub fn danger_button(text: impl Into<SharedString>) -> Div {
    button(text).bg(danger_bg()).text_color(danger())
}

/// Fila seleccionable de una lista (micrófonos).
pub fn list_row(is_selected: bool) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .h(px(30.))
        .px(px(10.))
        .rounded_lg()
        .bg(if is_selected { selected() } else { well() })
        .border_1()
        .border_color(if is_selected { accent() } else { border() })
}

/// Pista visual de una escala 0–100.
pub fn meter_fill(percent: u8) -> Div {
    let width = 8.0 + f32::from(percent.min(100)) * 1.6;
    div().h(px(6.)).w(px(width)).rounded_full().bg(accent())
}
