//! Ventana de ajustes: avatar, micrófono, nivel de audio y ventana.
//!
//! La composición sigue a `gentle-shell-gpui`: cabecera fija, cuerpo con
//! scroll y pie fijo. Sin botón "Guardar": cada control confirma su cambio, lo
//! persiste y avisa al resto de la aplicación. Todos los controles son piezas
//! de `shared::ui`; no hay componentes de terceros.

use std::time::Duration;

use gpui_kit::*;

use crate::app::AppState;
use crate::features::audio::AudioStatus;
use crate::features::audio::Vad;
use crate::features::audio::devices::DeviceEntry;
use crate::features::avatar::{self, AvatarSlot};
use crate::features::settings::Settings;
use crate::shared::ui;
use crate::window;

const MAX_DEVICE_ROWS: usize = 4;
/// Escala visible del medidor, en dBFS.
const METER_FLOOR_DB: f32 = -60.0;
/// Refresco del medidor mientras la ventana de ajustes existe.
const METER_FRAME: Duration = Duration::from_millis(45);

/// Eje que modifica un stepper de tamaño.
#[derive(Clone, Copy)]
enum SizeAxis {
    Width,
    Height,
}

/// Datos de un stepper de tamaño independiente.
struct SizeSpec {
    title: &'static str,
    description: &'static str,
    value: f32,
    down_id: &'static str,
    up_id: &'static str,
    down: f32,
    up: f32,
    axis: SizeAxis,
}

pub struct SettingsView {
    state: Entity<AppState>,
    body_scroll: ScrollHandle,
    /// Nivel suavizado que se dibuja; el audio solo guarda el instantáneo.
    display_db: f32,
    #[allow(dead_code)]
    observed: Subscription,
    #[allow(dead_code)]
    ticker: Task<()>,
}

impl SettingsView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let observed = cx.observe(&state, |_, _, cx| cx.notify());
        let ticker = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(METER_FRAME).await;
                let alive = this
                    .update(cx, |view, cx| {
                        let instant = view.state.read(cx).level_db();
                        if view.push_level(instant) {
                            cx.notify();
                        }
                    })
                    .is_ok();
                if !alive {
                    break;
                }
            }
        });

        Self {
            state,
            body_scroll: ScrollHandle::new(),
            display_db: METER_FLOOR_DB,
            observed,
            ticker,
        }
    }

    /// Suaviza la caída del medidor; las subidas se ven al instante.
    /// Devuelve `true` solo cuando el valor dibujado cambió lo suficiente.
    fn push_level(&mut self, instant: f32) -> bool {
        let instant = if instant.is_finite() {
            instant.clamp(METER_FLOOR_DB, 0.0)
        } else {
            METER_FLOOR_DB
        };
        let previous = self.display_db;
        if instant >= self.display_db {
            self.display_db = instant;
        } else {
            self.display_db += (instant - self.display_db) * 0.22;
        }
        (self.display_db - previous).abs() > 0.05
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

    fn apply_width(&self, width: f32, cx: &mut Context<Self>) {
        self.update_state(cx, move |state| state.set_width(width));
        window::resize_avatar(&self.state, cx);
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

    fn header(&self) -> Div {
        ui::screen_header().child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(32.))
                        .h(px(32.))
                        .rounded_md()
                        .bg(ui::primary())
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(ui::primary_fg())
                        .child("P"),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(ui::fg())
                                .child("PNGTuber Lite"),
                        )
                        .child(ui::hint("Overlay transparente · detección por RMS")),
                ),
        )
    }

    fn avatar_section(&self, settings: &Settings, cx: &mut Context<Self>) -> Div {
        ui::section(
            "Avatar",
            "Los dos estados PNG que se alternan cuando hablas.",
        )
        .child(
            ui::card()
                .p(px(6.))
                .child(Self::avatar_row(
                    AvatarSlot::Closed,
                    settings.avatar.closed.as_deref(),
                    cx,
                ))
                .child(ui::divider())
                .child(Self::avatar_row(
                    AvatarSlot::Open,
                    settings.avatar.open.as_deref(),
                    cx,
                )),
        )
    }

    fn avatar_row(slot: AvatarSlot, current: Option<&str>, cx: &mut Context<Self>) -> Div {
        let file = current
            .map(|path| path.rsplit('/').next().unwrap_or(path).to_string())
            .unwrap_or_else(|| "Sin imagen".to_string());
        let pick_id = match slot {
            AvatarSlot::Closed => "pick-closed",
            AvatarSlot::Open => "pick-open",
        };

        ui::row()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(36.))
                            .h(px(36.))
                            .rounded_md()
                            .bg(ui::muted_bg())
                            .border_1()
                            .border_color(ui::border())
                            .text_xs()
                            .text_color(ui::faint())
                            .child("PNG"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .min_w_0()
                            .child(ui::label(slot.title()))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(ui::faint())
                                    .truncate()
                                    .child(file),
                            ),
                    ),
            )
            .child(ui::button(pick_id, "Elegir").on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _: &MouseDownEvent, _window, cx| this.pick_png(slot, cx)),
            ))
    }

    fn audio_section(
        settings: &Settings,
        devices: &[DeviceEntry],
        devices_error: Option<&str>,
        status: &AudioStatus,
        audio_error: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Div {
        let enabled = settings.audio.enabled;

        let mut device_list = div().flex().flex_col().p(px(6.));
        device_list = device_list.child(Self::device_row(
            "device-default".to_string(),
            None,
            "Predeterminado del sistema".to_string(),
            settings.audio.device_id.is_none(),
            cx,
        ));
        for (index, entry) in devices.iter().take(MAX_DEVICE_ROWS).enumerate() {
            let selected = settings.audio.device_id.as_deref() == Some(entry.id.as_str());
            device_list = device_list.child(Self::device_row(
                format!("device-{index}"),
                Some(entry.id.clone()),
                entry.name.clone(),
                selected,
                cx,
            ));
        }
        if devices.is_empty() {
            device_list = device_list.child(
                div()
                    .px(px(10.))
                    .py(px(8.))
                    .text_xs()
                    .text_color(ui::faint())
                    .child(if devices_error.is_some() {
                        "No se encontraron micrófonos."
                    } else {
                        "Buscando micrófonos…"
                    }),
            );
        }

        let (status_text, status_color) = match status {
            AudioStatus::Stopped => ("Detección desactivada".to_string(), ui::faint()),
            AudioStatus::Starting => ("Conectando con el micrófono…".to_string(), ui::warning()),
            AudioStatus::Running(name) => (name.clone(), ui::success()),
            AudioStatus::Error(message) => (message.clone(), ui::danger()),
        };

        let mut card = ui::card()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap_4()
                    .p(px(16.))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(ui::label("Detección de voz"))
                            .child(ui::hint("Abre la boca cuando superas el umbral.")),
                    )
                    .child(ui::switch("audio-enabled", enabled).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseDownEvent, _, cx| {
                            let next = !this.state.read(cx).settings.audio.enabled;
                            this.update_state(cx, move |state| {
                                state.set_audio_enabled(next);
                            });
                        }),
                    )),
            )
            .child(ui::divider())
            .child(device_list)
            .child(ui::divider())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .px(px(16.))
                    .py(px(12.))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .min_w_0()
                            .child(ui::status_dot(status_color))
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_xs()
                                    .text_color(status_color)
                                    .child(status_text),
                            ),
                    )
                    .child(
                        ui::ghost_button("refresh-devices", "Actualizar").on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _: &MouseDownEvent, _, cx| this.refresh_devices(cx)),
                        ),
                    ),
            );

        if let Some(message) = audio_error {
            card = card.child(ui::notice(message.to_string(), ui::warning()).child(
                ui::ghost_button("dismiss-audio-error", "Descartar").on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _: &MouseDownEvent, _, cx| {
                        this.update_state(cx, |state| state.clear_audio_error());
                    }),
                ),
            ));
        }

        if let Some(message) = devices_error {
            card = card.child(ui::notice(message.to_string(), ui::danger()));
        }

        if devices.len() > MAX_DEVICE_ROWS {
            let hidden = devices.len() - MAX_DEVICE_ROWS;
            card = card.child(
                div()
                    .px(px(16.))
                    .pb(px(12.))
                    .text_xs()
                    .text_color(ui::faint())
                    .child(format!("{hidden} micrófonos más no listados")),
            );
        }

        card
    }

    fn device_row(
        row_id: String,
        id: Option<String>,
        name: String,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        ui::list_row(row_id, selected)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .flex_1()
                    .min_w_0()
                    .child(ui::radio(selected))
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .text_sm()
                            .text_color(if selected { ui::fg() } else { ui::muted() })
                            .child(name),
                    ),
            )
            .child(if selected {
                ui::chip("ACTIVO", ui::success())
            } else {
                div()
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                    let id = id.clone();
                    this.update_state(cx, move |state| state.set_device(id.clone()));
                }),
            )
    }

    fn audio_level_section(
        &self,
        settings: &Settings,
        status: &AudioStatus,
        mouth_open: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let value = settings.audio.sensitivity;
        let threshold = Vad::threshold_db(value);
        let running = matches!(status, AudioStatus::Running(_));
        let level = if running {
            db_fraction(self.display_db)
        } else {
            0.0
        };
        let threshold_fraction = db_fraction(threshold);
        let db_text = if running {
            format_db(self.display_db)
        } else {
            "—".to_string()
        };
        let (state_text, state_color) = match status {
            AudioStatus::Running(_) if mouth_open => ("HABLANDO", ui::success()),
            AudioStatus::Running(_) => ("SILENCIO", ui::faint()),
            AudioStatus::Starting => ("CONECTANDO", ui::warning()),
            AudioStatus::Error(_) => ("ERROR", ui::danger()),
            AudioStatus::Stopped => ("APAGADO", ui::faint()),
        };

        let lower = value.saturating_sub(5);
        let higher = value.saturating_add(5).min(100);

        let mut presets = ui::segmented();
        for (label, preset) in [("Baja", 25_u8), ("Media", 55), ("Alta", 80)] {
            presets = presets.child(
                ui::segment(("preset", preset as usize), label, value == preset).on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                        this.update_state(cx, move |state| state.set_sensitivity(preset));
                    }),
                ),
            );
        }

        ui::section(
            "Nivel de audio",
            "El umbral marca el punto exacto en que se abre la boca.",
        )
        .child(
            ui::card().child(
                ui::card_body()
                    .child(
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
                                    .gap_1()
                                    .child(ui::label("Nivel del micrófono"))
                                    .child(ui::hint(
                                        "Se actualiza en vivo mientras esta ventana está abierta.",
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_end()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(if mouth_open {
                                                ui::success()
                                            } else {
                                                ui::fg()
                                            })
                                            .child(db_text),
                                    )
                                    .child(ui::chip(state_text, state_color)),
                            ),
                    )
                    .child(ui::meter(level, threshold_fraction, mouth_open))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .child(ui::hint("−60 dB"))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_1()
                                    .child(ui::status_dot(ui::warning()))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(ui::warning())
                                            .child(format!("Umbral {}", format_db(threshold))),
                                    ),
                            )
                            .child(ui::hint("0 dB")),
                    )
                    .child(ui::divider())
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
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
                                            .gap_1()
                                            .child(ui::label("Sensibilidad"))
                                            .child(ui::hint(
                                                "Baja si la boca se abre sola; sube si no reacciona.",
                                            )),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_2()
                                            .child(ui::icon_button("sensitivity-down", "−").on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                                                    this.update_state(cx, move |state| {
                                                        state.set_sensitivity(lower);
                                                    });
                                                }),
                                            ))
                                            .child(ui::value_badge(format!("{value} / 100")))
                                            .child(ui::icon_button("sensitivity-up", "+").on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                                                    this.update_state(cx, move |state| {
                                                        state.set_sensitivity(higher);
                                                    });
                                                }),
                                            )),
                                    ),
                            )
                            .child(presets),
                    ),
            ),
        )
    }

    fn size_stepper(spec: SizeSpec, cx: &mut Context<Self>) -> Div {
        let SizeSpec {
            title,
            description,
            value,
            down_id,
            up_id,
            down,
            up,
            axis,
        } = spec;
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
                    .gap_1()
                    .child(ui::label(title))
                    .child(ui::hint(description)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(ui::icon_button(down_id, "−").on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _: &MouseDownEvent, _, cx| match axis {
                            SizeAxis::Width => this.apply_width(down, cx),
                            SizeAxis::Height => this.apply_height(down, cx),
                        }),
                    ))
                    .child(ui::value_badge(format!("{} px", value.round() as i32)))
                    .child(ui::icon_button(up_id, "+").on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _: &MouseDownEvent, _, cx| match axis {
                            SizeAxis::Width => this.apply_width(up, cx),
                            SizeAxis::Height => this.apply_height(up, cx),
                        }),
                    )),
            )
    }

    fn window_section(&self, settings: &Settings, cx: &mut Context<Self>) -> Div {
        let width = settings.window.width;
        let height = settings.window.height;
        let narrower = (width - 40.0).max(120.0);
        let wider = (width + 40.0).min(1200.0);
        let shorter = (height - 40.0).max(120.0);
        let taller = (height + 40.0).min(900.0);
        let topmost = settings.window.always_on_top;

        ui::section("Ventana", "Tamaño libre y visibilidad para OBS.").child(
            ui::card().child(
                ui::card_body()
                    .child(Self::size_stepper(
                        SizeSpec {
                            title: "Ancho",
                            description: "Píxeles lógicos; el PNG se adapta.",
                            value: width,
                            down_id: "width-down",
                            up_id: "width-up",
                            down: narrower,
                            up: wider,
                            axis: SizeAxis::Width,
                        },
                        cx,
                    ))
                    .child(ui::divider())
                    .child(Self::size_stepper(
                        SizeSpec {
                            title: "Alto",
                            description: "Píxeles lógicos; el PNG se adapta.",
                            value: height,
                            down_id: "height-down",
                            up_id: "height-up",
                            down: shorter,
                            up: taller,
                            axis: SizeAxis::Height,
                        },
                        cx,
                    ))
                    .child(ui::divider())
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(ui::label("En mi pantalla"))
                            .child(ui::hint(
                                "Oculto sigue renderizando para OBS. Atajo: Ctrl+Alt+P.",
                            ))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(ui::button("hide-avatar", "Ocultar").on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                            window::park_avatar(&this.state, cx);
                                        }),
                                    ))
                                    .child(
                                        ui::primary_button("show-avatar", "Mostrar").on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                                window::show_avatar(&this.state, cx);
                                            }),
                                        ),
                                    ),
                            ),
                    )
                    .child(ui::hint(
                        "Minimizar congela la captura de OBS; «Ocultar» la mantiene activa.",
                    ))
                    .child(ui::divider())
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .gap_4()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(ui::label("Siempre visible"))
                                    .child(ui::hint(
                                        "Mantiene el avatar por encima de las demás ventanas.",
                                    )),
                            )
                            .child(ui::switch("always-on-top", topmost).on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                    let next = !this.state.read(cx).settings.window.always_on_top;
                                    this.update_state(cx, move |state| {
                                        state.set_always_on_top(next);
                                    });
                                    this.apply_topmost(next, cx);
                                }),
                            )),
                    )
                    .child(ui::hint(
                        "Clic derecho sobre el avatar para volver a abrir estos ajustes.",
                    )),
            ),
        )
    }

    fn footer(&self, storage_error: Option<&str>, cx: &mut Context<Self>) -> Div {
        let mut footer = div()
            .flex()
            .flex_col()
            .gap_2()
            .flex_shrink_0()
            .px(px(24.))
            .py(px(14.))
            .border_t_1()
            .border_color(ui::border())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap_4()
                    .child(ui::hint(
                        "Se guarda solo. Minimiza la ventana; no se puede cerrar.",
                    ))
                    .child(
                        ui::destructive_button("quit-app", "Salir de la app").on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|_, _: &MouseDownEvent, _, cx| cx.quit()),
                        ),
                    ),
            );

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

/// Lleva un nivel en dBFS a la escala visible 0..=1.
fn db_fraction(db: f32) -> f32 {
    ((db.clamp(METER_FLOOR_DB, 0.0) - METER_FLOOR_DB) / -METER_FLOOR_DB).clamp(0.0, 1.0)
}

/// Formatea dBFS con el signo menos tipográfico.
fn format_db(db: f32) -> String {
    format!("{db:.0} dB").replace('-', "−")
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
            .bg(ui::window_bg())
            .text_color(ui::fg())
            .child(self.header())
            .child(
                div()
                    .id("settings-body")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.body_scroll)
                    .child(
                        div().flex().flex_col().items_center().p(px(24.)).child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_6()
                                .w_full()
                                .max_w(px(760.))
                                .child(self.avatar_section(&settings, cx))
                                .child(Self::audio_section(
                                    &settings,
                                    &devices,
                                    devices_error.as_deref(),
                                    &status,
                                    audio_error.as_deref(),
                                    cx,
                                ))
                                .child(self.audio_level_section(&settings, &status, mouth_open, cx))
                                .child(self.window_section(&settings, cx)),
                        ),
                    ),
            )
            .child(self.footer(storage_error.as_deref(), cx))
    }
}
