//! Punto de entrada: solo arranque y composicion.
//!
//! La regla del repo: `main.rs` no tiene logica ni vistas; registra assets,
//! fuentes y abre la ventana con `features::chat::ChatView`.

mod features;
mod shared;

use std::borrow::Cow;

use gpui_kit::*;
use shared::assets::AppAssets;

use features::chat::ChatView;

fn main() {
    application().with_assets(AppAssets).run(|cx| {
        // Temas y componentes del Kit, una sola vez.
        init(cx);

        // Fuente empaquetada: antes del primer frame
        // (patron de `gpui-kit.com/docs/fonts`).
        cx.text_system()
            .add_fonts(vec![
                Cow::Borrowed(
                    include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf").as_slice(),
                ),
                Cow::Borrowed(include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf").as_slice()),
            ])
            .expect("no se pudieron cargar las fuentes");

        // Ventana centrada en pantalla; el chat va centrado dentro.
        let bounds = Bounds::centered(None, size(px(1000.), px(700.)), cx);

        open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            cx,
            |_, cx| cx.new(|_| ChatView::new()),
        )
        .expect("no se pudo abrir la ventana");

        cx.activate(true);
    });
}
