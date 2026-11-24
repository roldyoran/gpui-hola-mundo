//! Fuente de recursos de la app: fondo, iconos Tabler y resto de assets.
//!
//! En GPUI un string como `"icons/send.svg"` nunca se lee del disco: se busca
//! por clave exacta en el `AssetSource` registrado. Aqui se sirven los bytes
//! empaquetados con `include_bytes!` y lo demas se delega a los iconos del Kit.

use std::borrow::Cow;

use anyhow::Result;
use gpui_kit::*;

pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let bytes: Option<&'static [u8]> = match path {
            "images/slava-jamm.jpg" => {
                Some(include_bytes!("../../assets/images/slava-jamm.jpg").as_slice())
            }
            "icons/send.svg" => Some(include_bytes!("../../assets/icons/send.svg").as_slice()),
            "icons/paperclip.svg" => {
                Some(include_bytes!("../../assets/icons/paperclip.svg").as_slice())
            }
            "icons/plus.svg" => Some(include_bytes!("../../assets/icons/plus.svg").as_slice()),
            "icons/settings.svg" => {
                Some(include_bytes!("../../assets/icons/settings.svg").as_slice())
            }
            "icons/copy.svg" => Some(include_bytes!("../../assets/icons/copy.svg").as_slice()),
            "icons/check.svg" => Some(include_bytes!("../../assets/icons/check.svg").as_slice()),
            _ => None,
        };
        match bytes {
            Some(data) => Ok(Some(Cow::Borrowed(data))),
            None => assets::Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        assets::Assets.list(path)
    }
}
