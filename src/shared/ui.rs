//! Tokens y piezas visuales compartidas por todos los features.
//!
//! Copiado de `gentle-shell-gpui` (`crates/app/src/shared/ui.rs`) y adaptado:
//! alli los tokens leen la paleta del tema activo; aqui son constantes porque
//! la app es de un solo tema oscuro sobre foto. Regla de la casa: ningun
//! feature inventa colores, bordes o radios; todo sale de este modulo.

use gpui_kit::*;

// ── Tokens ───────────────────────────────────────────────────────────────

/// Fondo del panel de chat.
pub fn panel() -> Rgba {
    rgba(0x141419f2)
}

/// Tarjetas y campos.
pub fn surface() -> Rgba {
    rgba(0x1e1e28ff)
}

/// Burbuja del usuario.
pub fn raised() -> Rgba {
    rgba(0x35334aff)
}

/// Huecos oscuros (composer, codigo).
pub fn well() -> Rgba {
    rgba(0x0e0e13ff)
}

/// Bordes sutiles.
pub fn border() -> Rgba {
    rgba(0xffffff1f)
}

/// Bordes que deben notarse (panel).
pub fn border_strong() -> Rgba {
    rgba(0xffffff3d)
}

/// Borde de campos de texto.
pub fn input_border() -> Rgba {
    rgba(0xffffff2e)
}

/// Texto principal.
pub fn fg() -> Rgba {
    rgb(0xf2f0eb)
}

/// Texto secundario.
pub fn muted() -> Rgba {
    rgb(0xc9c4bb)
}

/// Texto tenue (placeholders, pistas).
pub fn faint() -> Rgba {
    rgb(0x8a857c)
}

/// Acento de marca: rosa Gentle para enviar, avatar y focos.
pub fn accent() -> Rgba {
    rgb(0xe85d9e)
}

pub fn success() -> Rgba {
    rgb(0x7ee2a8)
}

pub fn info() -> Rgba {
    rgb(0x7ab8f5)
}

pub fn warning() -> Rgba {
    rgb(0xf5c97a)
}

pub fn danger() -> Rgba {
    rgb(0xf58a8a)
}

/// Familia monoespaciada de la app (empaquetada, ver `shared::assets`).
pub fn mono() -> SharedString {
    "JetBrains Mono".into()
}

/// Atajo con la tecla tal cual (`Ctrl+K` fuera de macOS, `⌘K` en macOS).
/// Copiado de `ui::shortcut` (alla coincide con los `key!()` de `app/mod.rs`).
pub fn shortcut(key: &str) -> String {
    if cfg!(target_os = "macos") {
        format!("⌘{key}")
    } else {
        format!("Ctrl+{key}")
    }
}

// ── Building blocks ──────────────────────────────────────────────────────
// Copiados de `ui.rs` de gentle-shell: misma forma, tokens propios.

/// La tarjeta estandar: paneles, burbujas, grupos.
pub fn card() -> Div {
    div()
        .flex()
        .flex_col()
        .rounded_xl()
        .bg(surface())
        .border_1()
        .border_color(border())
}

/// Recuadro oscuro para codigo y salidas.
pub fn well_box() -> Div {
    div()
        .flex()
        .flex_col()
        .rounded_lg()
        .bg(well())
        .border_1()
        .border_color(border())
        .font_family(mono())
        .text_sm()
}

/// Variantes de [`chip`]: la casa no admite colores sueltos, solo tokens.
pub enum ChipKind {
    Neutral,
    Success,
    Info,
    Accent,
}

/// Pildora pequena: estado, modelo, etiquetas.
pub fn chip(text: impl Into<SharedString>, kind: ChipKind) -> Div {
    let (text_color, bg, border_color) = match kind {
        ChipKind::Neutral => (muted(), rgba(0xffffff12), rgba(0xffffff1f)),
        ChipKind::Success => (success(), rgba(0x7ee2a81f), rgba(0x7ee2a838)),
        ChipKind::Info => (info(), rgba(0x7ab8f51f), rgba(0x7ab8f538)),
        ChipKind::Accent => (accent(), rgba(0xe85d9e1f), rgba(0xe85d9e38)),
    };
    div()
        .px_1p5()
        .py(px(1.))
        .rounded_md()
        .text_sm()
        .text_color(text_color)
        .bg(bg)
        .border_1()
        .border_color(border_color)
        .child(text.into())
}

/// Tecla de atajo (`esc`, `↵`).
pub fn kbd(text: impl Into<SharedString>) -> Div {
    div()
        .min_w(px(20.))
        .h(px(20.))
        .px_1p5()
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .border_1()
        .border_color(border_strong())
        .bg(surface())
        .font_family(mono())
        .text_sm()
        .text_color(muted())
        .child(text.into())
}

/// Micro encabezado en mayusculas para secciones.
pub fn section_label(text: impl Into<SharedString>) -> Div {
    div()
        .text_sm()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(faint())
        .child(text.into().to_uppercase())
}

/// Fila de cabecera de los paneles: icono, titulo y acciones.
pub fn screen_header() -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .px_5()
        .py_3()
        .gap_3()
        .border_b_1()
        .border_color(border())
}

/// Marco de campo de texto (altura, radio y borde unicos en la app).
pub fn field() -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .h(px(34.))
        .px_3()
        .rounded_lg()
        .bg(well())
        .border_1()
        .border_color(input_border())
}
