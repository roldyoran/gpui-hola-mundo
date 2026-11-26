//! Estado global de la aplicación y bomba de eventos de audio.
//!
//! `AppState` es la única fuente de verdad: ajustes persistidos, estado de la
//! boca y handles de las dos ventanas. Las vistas lo observan y se redibujan
//! solo cuando cambia. El audio corre en su propio hilo y publica eventos por
//! un canal acotado; aquí se drenan en una tarea de GPUI, sin polling.

use async_channel::Receiver;
use gpui_kit::*;

use crate::features::audio::devices::DeviceEntry;
use crate::features::audio::{AudioConfig, AudioEvent, AudioService, AudioStatus};
use crate::features::avatar::AvatarSlot;
use crate::features::settings::{Paths, Settings};

/// Estado compartido por la ventana del avatar y la de ajustes.
pub struct AppState {
    pub settings: Settings,
    pub paths: Paths,
    pub audio: AudioService,
    pub mouth_open: bool,
    pub audio_status: AudioStatus,
    pub audio_error: Option<String>,
    pub devices: Vec<DeviceEntry>,
    pub devices_error: Option<String>,
    pub storage_error: Option<String>,
    /// Incrementa cuando cambian los PNG; la vista recarga sus texturas.
    pub avatar_generation: u64,
    pub settings_window: Option<AnyWindowHandle>,
    pub avatar_window: Option<AnyWindowHandle>,
    #[allow(dead_code)]
    pump: Task<()>,
}

impl AppState {
    /// Crea el estado, arranca el hilo de audio y engancha la bomba de eventos.
    pub fn new(cx: &mut Context<Self>, paths: Paths, settings: Settings) -> Self {
        let (events_tx, events_rx) = async_channel::bounded(8);
        let audio = AudioService::start(events_tx);
        let pump = cx.spawn(async move |this, cx| {
            drain_audio(events_rx, this, cx).await;
        });

        let state = Self {
            settings,
            paths,
            audio,
            mouth_open: false,
            audio_status: AudioStatus::Stopped,
            audio_error: None,
            devices: Vec::new(),
            devices_error: None,
            storage_error: None,
            avatar_generation: 0,
            settings_window: None,
            avatar_window: None,
            pump,
        };
        state.apply_audio();
        state.refresh_devices();
        state
    }

    /// Aplica la configuración de audio vigente al hilo dueño del stream.
    pub fn apply_audio(&self) {
        self.audio.apply(AudioConfig {
            enabled: self.settings.audio.enabled,
            device_id: self.settings.audio.device_id.clone(),
            sensitivity: self.settings.audio.sensitivity,
        });
    }

    /// Pide al hilo de audio la lista de micrófonos.
    pub fn refresh_devices(&self) {
        self.audio.refresh_devices();
    }

    /// Guarda los ajustes de forma atómica y deja constancia de un fallo.
    pub fn persist(&mut self) {
        match self.settings.save(&self.paths) {
            Ok(()) => self.storage_error = None,
            Err(error) => self.storage_error = Some(format!("No se pudo guardar: {error:#}")),
        }
    }

    pub fn set_audio_enabled(&mut self, enabled: bool) {
        self.settings.audio.enabled = enabled;
        self.persist();
        self.apply_audio();
    }

    pub fn set_device(&mut self, device_id: Option<String>) {
        self.settings.audio.device_id = device_id;
        self.persist();
        self.apply_audio();
    }

    pub fn set_sensitivity(&mut self, sensitivity: u8) {
        self.settings.audio.sensitivity = sensitivity.min(100);
        self.persist();
        self.apply_audio();
    }

    pub fn set_always_on_top(&mut self, always_on_top: bool) {
        self.settings.window.always_on_top = always_on_top;
        self.persist();
    }

    /// Cambia la altura y conserva la proporción actual de la ventana.
    pub fn set_height(&mut self, height: f32) {
        let height = height.clamp(120.0, 900.0);
        let ratio = if self.settings.window.height > 0.0 {
            self.settings.window.width / self.settings.window.height
        } else {
            1.0
        };
        self.settings.window.height = height;
        self.settings.window.width = (height * ratio).clamp(120.0, 1200.0);
        self.persist();
    }

    pub fn set_avatar(&mut self, slot: AvatarSlot, relative: String) {
        match slot {
            AvatarSlot::Closed => self.settings.avatar.closed = Some(relative),
            AvatarSlot::Open => self.settings.avatar.open = Some(relative),
        }
        self.avatar_generation = self.avatar_generation.wrapping_add(1);
        self.persist();
    }

    /// Guarda la posición y el tamaño reales que dejó el arrastre.
    pub fn set_bounds(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.settings.window.x = Some(x);
        self.settings.window.y = Some(y);
        self.settings.window.width = width;
        self.settings.window.height = height;
        self.persist();
    }

    fn on_audio_event(&mut self, event: AudioEvent) -> bool {
        match event {
            AudioEvent::Mouth(open) => {
                let changed = self.mouth_open != open;
                self.mouth_open = open;
                changed
            }
            AudioEvent::Status(status) => {
                if !matches!(&status, AudioStatus::Error(_)) {
                    self.audio_error = None;
                }
                let changed = self.audio_status != status;
                self.audio_status = status;
                changed
            }
            AudioEvent::Devices(list) => {
                self.devices = list;
                self.devices_error = None;
                true
            }
            AudioEvent::DevicesError(message) => {
                self.devices_error = Some(message);
                true
            }
            AudioEvent::Error(message) => {
                self.audio_error = Some(message);
                true
            }
        }
    }
}

/// Drena eventos de audio hasta que se cierre el canal o muera el estado.
async fn drain_audio(events: Receiver<AudioEvent>, this: WeakEntity<AppState>, cx: &mut AsyncApp) {
    while let Ok(event) = events.recv().await {
        let alive = this
            .update(cx, |state, cx| {
                if state.on_audio_event(event) {
                    cx.notify();
                }
            })
            .is_ok();
        if !alive {
            break;
        }
    }
}
