use std::borrow::Cow;

use anyhow::Result;
use gpui_kit::component::label::Label;
use gpui_kit::*;

// Fuente de recursos de la app: iconos por defecto del Kit + fondo empaquetado.
// Un string como "images/slava-jamm.jpg" NUNCA se lee del disco en GPUI:
// se busca por clave exacta en el AssetSource registrado.
struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path == "images/slava-jamm.jpg" {
            Ok(Some(
                include_bytes!("../assets/images/slava-jamm.jpg")
                    .as_slice()
                    .into(),
            ))
        } else {
            assets::Assets.load(path)
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        assets::Assets.list(path)
    }
}

// Vista raiz: no guarda estado, solo dibuja.
// GPUI llama a `render` en cada frame cuando el estado cambia.
struct HolaMundo;

impl Render for HolaMundo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .size_full()
            .bg(rgb(0x1e1e2e))
            // Fondo: foto a pantalla completa, recortada con Cover.
            .child(
                img("images/slava-jamm.jpg")
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h_full()
                    .object_fit(ObjectFit::Cover),
            )
            // Velo oscuro ~65% para que el texto blanco siempre lea bien
            // (la foto es muy clara arriba).
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h_full()
                    .bg(rgba(0x000000a6)),
            )
            // Contenido centrado encima del fondo.
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .child(
                        Label::new("Hola, mundo!")
                            .font_family("JetBrains Mono")
                            .text_3xl()
                            .text_color(rgb(0xffffff)),
                    )
                    .child(
                        div()
                            .font_family("JetBrains Mono")
                            .text_sm()
                            .opacity(0.85)
                            .text_color(rgb(0xffffff))
                            .child("GPUI + gpui-kit — texto centrado de prueba"),
                    ),
            )
    }
}

fn main() {
    application()
        .with_assets(AppAssets)
        .run(|cx| {
            // Inicializa temas y capas del Kit una sola vez.
            init(cx);

            // Fuente empaquetada: se registra ANTES del primer frame,
            // segun la guia oficial (gpui-kit.com/docs/fonts).
            // El nombre "JetBrains Mono" es el family name interno del TTF.
            cx.text_system()
                .add_fonts(vec![
                    Cow::Borrowed(
                        include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf")
                            .as_slice(),
                    ),
                    Cow::Borrowed(
                        include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf")
                            .as_slice(),
                    ),
                ])
                .expect("no se pudieron cargar las fuentes");

            // Ventana de 800x600 centrada en la pantalla.
            // El texto ademas va centrado dentro de la ventana
            // con items_center + justify_center.
            let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

            open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                cx,
                |_, cx| cx.new(|_| HolaMundo),
            )
            .expect("no se pudo abrir la ventana");

            cx.activate(true);
        });
}
