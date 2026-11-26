//! Ventana de ajustes: avatar, micrófono, sensibilidad y ventana.
//!
//! Sin botón "Guardar": cada control confirma su cambio, lo persiste y avisa
//! al resto de la aplicación. Todos los controles son piezas propias de
//! `shared::ui`; no hay componentes de terceros.

use gpui_kit::*;

use crate::app::AppState;
use crate::features::audio::AudioStatus;
use crate::features::audio::devices::DeviceEntry;
use crate::features::avatar::{self, AvatarSlot};
use crate::features::settings::Settings;
use crate::shared::ui;
use crate::window;

const MAX_DEVICE_ROWS: usize = 4;

pub struct SettingsView {
    state: Entity<AppState>,
    #[allow(dead_code)]
    observed: Subscription,
}

impl SettingsView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let observed = cx.observe(&state, |_, _, cx| cx.notify());
        Self { state, observed }
    }

    fn update_state(&self, cx: &mut Context<Self>, update: impl FnOnce(&mut AppState)) {
        self.state.update(cx, |state, cx| {
            update(state);
            cx.notify();
        });
        cx.notify();
    }

    fn refresh_devices(&self, cx: &mut Context<Self>) {
        self.update_state(cx, |state| state.refresh_devices());
    }

    fn pick_png(&mut self, slot: AvatarSlot, cx: &mut Context<Self>) {
        let paths = self.state.read(cx).paths.clone();
        let Some(source) = rfd::FileDialog::new()
            .add_filter("PNG", &["png"])
            .pick_file()
        else {
            return;
        };

        match avatar::import_png(&paths, &source, slot) {
            Ok(relative) => {
                self.update_state(cx, move |state| state.set_avatar(slot, relative));
            }
            Err(error) => {
                let message = format!("{error:#}");
                self.update_state(cx, move |state| {
                    state.storage_error = Some(message);
                });
            }
        }
    }

    fn apply_height(&self, height: f32, cx: &mut Context<Self>) {
        self.update_state(cx, move |state| state.set_height(height));
        window::resize_avatar(&self.state, cx);
    }

    fn apply_topmost(&self, topmost: bool, cx: &mut Context<Self>) {
        let handle = self.state.read(cx).avatar_window;
        if let Some(handle) = handle {
            let _ = handle.update(cx, |_, window, _| {
                crate::win32::set_topmost(window, topmost);
            });
        }
    }

    fn header(&self, cx: &mut Context<Self>) -> Div {
        ui::screen_header()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(ui::fg())
                            .child("PNGTuber Lite"),
                    )
                    .child(ui::hint("Overlay transparente · VAD por RMS")),
            )
            .child(
                ui::danger_button("Salir")
                    .on_click(cx.listener(|_, _: &ClickEvent, _, cx| cx.quit())),
            )
    }

    fn avatar_section(&self, settings: &Settings, cx: &mut Context<Self>) -> Div {
        ui::card()
            .child(ui::section_label("Avatar"))
            .child(Self::avatar_row(
                AvatarSlot::Closed,
                settings.avatar.closed.as_deref(),
                cx,
            ))
            .child(Self::avatar_row(
                AvatarSlot::Open,
                settings.avatar.open.as_deref(),
                cx,
            ))
    }

    fn avatar_row(slot: AvatarSlot, current: Option<&str>, cx: &mut Context<Self>) -> Div {
        let file = current
            .map(|path| path.rsplit('/').next().unwrap_or(path).to_string())
            .unwrap_or_else(|| "Sin imagen".to_string());

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap_3()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(ui::label(slot.title()))
                    .child(div().text_xs().text_color(ui::faint()).child(file)),
            )
            .child(ui::button("Elegir…").on_click(
                cx.listener(move |this, _: &ClickEvent, _window, cx| this.pick_png(slot, cx)),
            ))
    }

    fn audio_section(
        settings: &Settings,
        devices: &[DeviceEntry],
        devices_error: Option<&str>,
        mouth_open: bool,
        status: &AudioStatus,
        error: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Div {
        let enabled = settings.audio.enabled;

        let mut device_list = div().flex().flex_col().gap_2();
        device_list = device_list.child(Self::device_row(
            None,
            "Predeterminado del sistema".to_string(),
            settings.audio.device_id.is_none(),
            cx,
        ));
        for entry in devices.iter().take(MAX_DEVICE_ROWS) {
            let selected = settings.audio.device_id.as_deref() == Some(entry.id.as_str());
            device_list = device_list.child(Self::device_row(
                Some(entry.id.clone()),
                entry.name.clone(),
                selected,
                cx,
            ));
        }
        if devices.is_empty() {
            device_list = device_list.child(div().text_xs().text_color(ui::faint()).child(
                if devices_error.is_some() {
                    "Sin micrófonos"
                } else {
                    "Buscando micrófonos…"
                },
            ));
        }

        let (status_text, status_color) = match status {
            AudioStatus::Stopped => ("Detección desactivada".to_string(), ui::faint()),
            AudioStatus::Starting => ("Conectando con el micrófono…".to_string(), ui::warning()),
            AudioStatus::Running(name) => (format!("Escuchando: {name}"), ui::success()),
            AudioStatus::Error(message) => (format!("Error: {message}"), ui::danger()),
        };

        let mouth_color = if mouth_open {
            ui::accent()
        } else {
            ui::faint()
        };
        let mouth_bg = if mouth_open {
            ui::selected()
        } else {
            ui::well()
        };

        let mut card = ui::card()
            .child(ui::section_label("Micrófono"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(ui::label("Detección de voz"))
                    .child(
                        ui::button(if enabled { "Activada" } else { "Desactivada" }).on_click(
                            cx.listener(|this, _: &ClickEvent, _, cx| {
                                let next = !this.state.read(cx).settings.audio.enabled;
                                this.update_state(cx, move |state| {
                                    state.set_audio_enabled(next);
                                });
                            }),
                        ),
                    ),
            )
            .child(device_list);

        card =
            card.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px(px(8.))
                                    .py(px(2.))
                                    .rounded_md()
                                    .bg(mouth_bg)
                                    .text_xs()
                                    .text_color(mouth_color)
                                    .child(if mouth_open { "HABLANDO" } else { "SILENCIO" }),
                            )
                            .child(div().text_xs().text_color(status_color).child(status_text)),
                    )
                    .child(ui::button("Actualizar").on_click(
                        cx.listener(|this, _: &ClickEvent, _, cx| this.refresh_devices(cx)),
                    )),
            );

        if let Some(message) = error.or(devices_error) {
            card = card.child(
                div()
                    .text_xs()
                    .text_color(ui::danger())
                    .child(message.to_string()),
            );
        }

        if devices.len() > MAX_DEVICE_ROWS {
            let hidden = devices.len() - MAX_DEVICE_ROWS;
            card = card.child(
                div()
                    .text_xs()
                    .text_color(ui::faint())
                    .child(format!("{hidden} micrófonos más no listados")),
            );
        }

        card
    }

    fn device_row(id: Option<String>, name: String, selected: bool, cx: &mut Context<Self>) -> Div {
        ui::list_row(selected)
            .child(
                div()
                    .text_sm()
                    .text_color(if selected { ui::fg() } else { ui::muted() })
                    .child(name),
            )
            .child(div().text_xs().text_color(ui::accent()).child(if selected {
                "ACTIVO"
            } else {
                ""
            }))
            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                let id = id.clone();
                this.update_state(cx, move |state| state.set_device(id.clone()));
            }))
    }

    fn sensitivity_section(&self, settings: &Settings, cx: &mut Context<Self>) -> Div {
        let value = settings.audio.sensitivity;
        let lower = value.saturating_sub(5);
        let higher = value.saturating_add(5).min(100);

        ui::card()
            .child(ui::section_label("Sensibilidad"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(ui::button("−").on_click(cx.listener(
                        move |this, _: &ClickEvent, _, cx| {
                            this.update_state(cx, move |state| {
                                state.set_sensitivity(lower);
                            });
                        },
                    )))
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .text_color(ui::fg())
                            .child(format!("{value} / 100")),
                    )
                    .child(ui::button("+").on_click(cx.listener(
                        move |this, _: &ClickEvent, _, cx| {
                            this.update_state(cx, move |state| {
                                state.set_sensitivity(higher);
                            });
                        },
                    ))),
            )
            .child(
                div()
                    .h(px(6.))
                    .w_full()
                    .rounded_full()
                    .bg(ui::well())
                    .child(ui::meter_fill(value)),
            )
            .child(ui::hint(
                "Baja si la boca se abre sola; sube si no reacciona a tu voz.",
            ))
    }

    fn window_section(&self, settings: &Settings, cx: &mut Context<Self>) -> Div {
        let height = settings.window.height;
        let shorter = (height - 40.0).max(120.0);
        let taller = (height + 40.0).min(900.0);
        let topmost = settings.window.always_on_top;

        ui::card()
            .child(ui::section_label("Ventana"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(ui::label("Tamaño"))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(ui::button("−").on_click(cx.listener(
                                move |this, _: &ClickEvent, _, cx| this.apply_height(shorter, cx),
                            )))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(ui::fg())
                                    .child(format!("{} px", height.round() as i32)),
                            )
                            .child(ui::button("+").on_click(cx.listener(
                                move |this, _: &ClickEvent, _, cx| this.apply_height(taller, cx),
                            ))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(ui::label("Siempre visible"))
                    .child(
                        ui::button(if topmost { "Activado" } else { "Desactivado" }).on_click(
                            cx.listener(|this, _: &ClickEvent, _, cx| {
                                let next = !this.state.read(cx).settings.window.always_on_top;
                                this.update_state(cx, move |state| {
                                    state.set_always_on_top(next);
                                });
                                this.apply_topmost(next, cx);
                            }),
                        ),
                    ),
            )
            .child(ui::hint(
                "Clic derecho sobre el avatar para volver a abrir estos ajustes.",
            ))
    }

    fn footer(&self, storage_error: Option<&str>) -> Div {
        let mut footer = div().flex().flex_col().gap_2().child(ui::hint(
            "Los cambios se guardan solos. Cerrar esta ventana no cierra el avatar.",
        ));

        if let Some(message) = storage_error {
            footer = footer.child(
                div()
                    .text_xs()
                    .text_color(ui::danger())
                    .child(message.to_string()),
            );
        }
        footer
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (settings, mouth_open, status, audio_error, devices, devices_error, storage_error) = {
            let state = self.state.read(cx);
            (
                state.settings.clone(),
                state.mouth_open,
                state.audio_status.clone(),
                state.audio_error.clone(),
                state.devices.clone(),
                state.devices_error.clone(),
                state.storage_error.clone(),
            )
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(ui::panel())
            .text_color(ui::fg())
            .child(self.header(cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p(px(20.))
                    .flex_1()
                    .child(self.avatar_section(&settings, cx))
                    .child(Self::audio_section(
                        &settings,
                        &devices,
                        devices_error.as_deref(),
                        mouth_open,
                        &status,
                        audio_error.as_deref(),
                        cx,
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_3()
                            .child(self.sensitivity_section(&settings, cx).flex_1())
                            .child(self.window_section(&settings, cx).flex_1()),
                    )
                    .child(self.footer(storage_error.as_deref())),
            )
    }
}
