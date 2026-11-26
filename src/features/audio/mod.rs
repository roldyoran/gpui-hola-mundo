//! Servicio de audio: hilo dueño del stream y eventos hacia la UI.
//!
//! El hilo está dormido en `recv_blocking` hasta que llega un comando. Cuando
//! llega, reconcilia la configuración (apagar, encender, cambiar dispositivo o
//! solo actualizar la sensibilidad) o enumera micrófonos. **Ninguna llamada a
//! cpal ocurre en el hilo de GPUI.**

pub mod devices;
pub mod stream;
pub mod vad;

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU32, Ordering};

use async_channel::{Receiver, Sender};

use devices::DeviceEntry;

pub use vad::Vad;

/// Nivel en reposo: por debajo de cualquier señal real.
const SILENCE_DB: f32 = f32::NEG_INFINITY;

fn silent_level() -> u32 {
    SILENCE_DB.to_bits()
}

/// Lo mínimo que el hilo de audio necesita saber de los ajustes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioConfig {
    pub enabled: bool,
    pub device_id: Option<String>,
    pub sensitivity: u8,
}

/// Estado visible del micrófono en la ventana de ajustes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioStatus {
    Stopped,
    Starting,
    Running(String),
    Error(String),
}

/// Eventos del hilo de audio hacia el hilo de GPUI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioEvent {
    Mouth(bool),
    Status(AudioStatus),
    Devices(Vec<DeviceEntry>),
    DevicesError(String),
    Error(String),
}

enum Command {
    Apply(AudioConfig),
    RefreshDevices,
}

/// Handle al hilo de audio. Se puede clonar el `Sender` internamente.
pub struct AudioService {
    commands: Sender<Command>,
    /// Último RMS en dBFS (bits de `f32`); la UI lo lee sin eventos.
    level: Arc<AtomicU32>,
}

impl AudioService {
    /// Arranca el hilo y devuelve el handle para enviarle comandos.
    pub fn start(events: Sender<AudioEvent>) -> Self {
        let (commands, receiver) = async_channel::unbounded();
        let level = Arc::new(AtomicU32::new(silent_level()));
        let thread_level = level.clone();
        let _ = std::thread::Builder::new()
            .name("pngtuber-audio".to_string())
            .spawn(move || run(receiver, events, thread_level));
        Self { commands, level }
    }

    /// Aplica una configuración completa (idempotente).
    pub fn apply(&self, config: AudioConfig) {
        let _ = self.commands.try_send(Command::Apply(config));
    }

    /// Pide al hilo de audio que vuelva a enumerar los micrófonos.
    pub fn refresh_devices(&self) {
        let _ = self.commands.try_send(Command::RefreshDevices);
    }

    /// Nivel RMS instantáneo en dBFS; `−∞` cuando no hay captura.
    pub fn level_db(&self) -> f32 {
        f32::from_bits(self.level.load(Ordering::Relaxed))
    }
}

fn run(commands: Receiver<Command>, events: Sender<AudioEvent>, level: Arc<AtomicU32>) {
    let sensitivity = Arc::new(AtomicU8::new(55));
    let mut active: Option<stream::ActiveStream> = None;
    let mut current: Option<AudioConfig> = None;

    while let Ok(command) = commands.recv_blocking() {
        let config = match command {
            Command::RefreshDevices => {
                match devices::list_input_devices() {
                    Ok(list) => {
                        let _ = events.try_send(AudioEvent::Devices(list));
                    }
                    Err(message) => {
                        let _ = events.try_send(AudioEvent::DevicesError(message));
                    }
                }
                continue;
            }
            Command::Apply(config) => config,
        };

        let healthy = active.is_some() || !config.enabled;
        if current.as_ref() == Some(&config) && healthy {
            continue;
        }

        sensitivity.store(config.sensitivity.min(100), Ordering::Relaxed);

        if !config.enabled {
            if active.take().is_some() {
                let _ = events.try_send(AudioEvent::Mouth(false));
            }
            level.store(silent_level(), Ordering::Relaxed);
            let _ = events.try_send(AudioEvent::Status(AudioStatus::Stopped));
            current = Some(config);
            continue;
        }

        let device_changed = current
            .as_ref()
            .is_none_or(|previous| previous.device_id != config.device_id);
        let was_disabled = current.as_ref().is_none_or(|previous| !previous.enabled);

        if active.is_none() || device_changed || was_disabled {
            active = None;
            level.store(silent_level(), Ordering::Relaxed);
            let _ = events.try_send(AudioEvent::Mouth(false));
            let _ = events.try_send(AudioEvent::Status(AudioStatus::Starting));

            match stream::open(&config, sensitivity.clone(), level.clone(), events.clone()) {
                Ok(opened) => {
                    let name = opened.device_name.clone();
                    active = Some(opened);
                    let _ = events.try_send(AudioEvent::Status(AudioStatus::Running(name)));
                }
                Err(message) => {
                    level.store(silent_level(), Ordering::Relaxed);
                    let _ = events.try_send(AudioEvent::Status(AudioStatus::Error(message)));
                }
            }
        }

        current = Some(config);
    }
}
