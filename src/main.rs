//! Punto de entrada: arranque y composición, sin lógica.
//!
//! Orden: rutas -> ajustes -> estado global -> ventana del avatar. Si todavía
//! no hay PNG configurados, se abre también la ventana de ajustes.

use gpui_kit::*;

use pngtuber_lite::app::AppState;
use pngtuber_lite::features::avatar::AvatarView;
use pngtuber_lite::features::settings::{Paths, Settings};
use pngtuber_lite::window;

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);

        let paths = Paths::discover();
        let settings = Settings::load(&paths);

        let needs_setup = settings.avatar.closed.is_none() || settings.avatar.open.is_none();
        let always_on_top = settings.window.always_on_top;
        let avatar_bounds = window::avatar_bounds(&settings, cx);

        let state = cx.new(|cx| AppState::new(cx, paths, settings));
        let avatar_state = state.clone();
        let avatar_view = cx.new(|cx| AvatarView::new(avatar_state, cx));
        let avatar_window = cx
            .open_window(window::avatar_options(avatar_bounds), move |_, _| {
                avatar_view
            })
            .expect("no se pudo abrir la ventana del avatar");
        let avatar_window: AnyWindowHandle = avatar_window.into();

        state.update(cx, |state, _| {
            state.avatar_window = Some(avatar_window);
        });

        let _ = avatar_window.update(cx, |_, window, _| {
            pngtuber_lite::win32::set_topmost(window, always_on_top);
        });

        if needs_setup {
            let _ = window::open_settings(state.clone(), cx);
        }
    });
}
