//! Avatar: validación/copia de PNG y decodificación en memoria.
//!
//! Los bytes se copian al directorio de datos para que mover el archivo
//! original no rompa la configuración. La decodificación ocurre una sola vez
//! por archivo: `img(Arc<Image>)` reutiliza la entrada de caché de GPUI.

pub mod view;

pub use view::AvatarView;

use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use gpui_kit::{Image, ImageFormat};

use crate::features::settings::Paths;

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Los dos estados de boca.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvatarSlot {
    Closed,
    Open,
}

impl AvatarSlot {
    pub fn file_name(self) -> &'static str {
        match self {
            Self::Closed => "closed.png",
            Self::Open => "open.png",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Closed => "Boca cerrada",
            Self::Open => "Boca abierta",
        }
    }
}

/// Comprueba la firma PNG; no decodifica (eso lo hace GPUI al renderizar).
pub fn is_png(bytes: &[u8]) -> bool {
    bytes.len() > PNG_SIGNATURE.len() && bytes.starts_with(&PNG_SIGNATURE)
}

/// Valida y copia un PNG al directorio de datos. Devuelve la ruta relativa.
pub fn import_png(paths: &Paths, source: &Path, slot: AvatarSlot) -> Result<String> {
    let bytes =
        fs::read(source).with_context(|| format!("no se pudo leer {}", source.display()))?;
    if !is_png(&bytes) {
        bail!("el archivo no es un PNG válido");
    }
    paths.ensure()?;
    let relative = format!("avatars/{}", slot.file_name());
    fs::write(paths.dir.join(&relative), &bytes)
        .with_context(|| "no se pudo copiar el PNG al directorio de datos")?;
    Ok(relative)
}

/// Decodifica un PNG en memoria. `None` si no es un PNG reconocible.
pub fn decode_png(bytes: &[u8]) -> Option<Arc<Image>> {
    if !is_png(bytes) {
        return None;
    }
    Some(Arc::new(Image::from_bytes(
        ImageFormat::Png,
        bytes.to_vec(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PNG 1×1 transparente mínimo.
    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn acepta_un_png_real() {
        assert!(is_png(TINY_PNG));
    }

    #[test]
    fn rechaza_otros_formatos() {
        assert!(!is_png(b"GIF89a"));
        assert!(!is_png(b""));
    }

    #[test]
    fn los_slots_tienen_nombres_estables() {
        assert_eq!(AvatarSlot::Closed.file_name(), "closed.png");
        assert_eq!(AvatarSlot::Open.file_name(), "open.png");
    }
}
