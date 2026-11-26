#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Punto de entrada: arranque y composición, sin lógica.
//!
//! El overlay **no** se crea al arrancar: la app abre siempre ajustes con la
//! previsualización. La ventana transparente nace la primera vez que el usuario
//! pulsa **Mostrar**.
//!
//! En release el binario usa el subsistema `windows`: al abrir el `.exe` no
//! aparece ninguna consola. En debug se conserva para ver los mensajes.

use gpui_kit::*;

use pngtuber_lite::app::AppState;
use pngtuber_lite::features::settings::{Paths, Settings};
use pngtuber_lite::window;

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);

        let paths = Paths::discover();
        let settings = Settings::load(&paths);

        let state = cx.new(|cx| AppState::new(cx, paths, settings));
        let _ = window::open_settings(state, cx);
    });
}
