//! Ajustes persistentes y rutas de datos.
//!
//! Esquema v1, sin migraciones: cualquier versión distinta o JSON corrupto
//! vuelve a los valores por defecto. La escritura es atómica (temporal +
//! `rename`, que en Windows reemplaza el destino).

pub mod view;

pub use view::SettingsView;

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Versión del esquema de `settings.json`. Solo existe la 1.
pub const SCHEMA_VERSION: u32 = 1;

const DEFAULT_SIZE: f32 = 320.0;
const MIN_SIZE: f32 = 120.0;
const MAX_WIDTH: f32 = 1200.0;
const MAX_HEIGHT: f32 = 900.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub schema_version: u32,
    pub audio: AudioSettings,
    pub avatar: AvatarSettings,
    pub window: WindowSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            audio: AudioSettings::default(),
            avatar: AvatarSettings::default(),
            window: WindowSettings::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioSettings {
    pub enabled: bool,
    /// `DeviceId` de cpal serializado con `Display`; `None` = predeterminado.
    pub device_id: Option<String>,
    /// 0 = poco sensible (−20 dBFS) … 100 = muy sensible (−60 dBFS).
    pub sensitivity: u8,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            device_id: None,
            sensitivity: 55,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AvatarSettings {
    /// Ruta relativa al directorio de datos (`avatars/closed.png`).
    pub closed: Option<String>,
    pub open: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowSettings {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub width: f32,
    pub height: f32,
    pub always_on_top: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: DEFAULT_SIZE,
            height: DEFAULT_SIZE,
            always_on_top: true,
        }
    }
}

impl Settings {
    /// Lee los ajustes; cualquier problema devuelve los valores por defecto.
    pub fn load(paths: &Paths) -> Self {
        let Ok(text) = fs::read_to_string(&paths.settings) else {
            return Self::default();
        };
        let Ok(settings) = serde_json::from_str::<Self>(&text) else {
            return Self::default();
        };
        if settings.schema_version != SCHEMA_VERSION {
            return Self::default();
        }
        let mut settings = settings;
        settings.sanitize();
        settings
    }

    /// Escribe `settings.json` de forma atómica.
    pub fn save(&self, paths: &Paths) -> Result<()> {
        paths.ensure()?;
        let json = serde_json::to_vec_pretty(self)?;
        let tmp = paths.settings.with_extension("json.tmp");
        fs::write(&tmp, json)?;
        fs::rename(&tmp, &paths.settings)?;
        Ok(())
    }

    /// Corrige valores fuera de rango para que la ventana siempre sea válida.
    fn sanitize(&mut self) {
        self.audio.sensitivity = self.audio.sensitivity.min(100);

        if !self.window.width.is_finite() {
            self.window.width = DEFAULT_SIZE;
        }
        if !self.window.height.is_finite() {
            self.window.height = DEFAULT_SIZE;
        }
        self.window.width = self.window.width.clamp(MIN_SIZE, MAX_WIDTH);
        self.window.height = self.window.height.clamp(MIN_SIZE, MAX_HEIGHT);

        if self.window.x.is_some_and(|x| !x.is_finite()) {
            self.window.x = None;
        }
        if self.window.y.is_some_and(|y| !y.is_finite()) {
            self.window.y = None;
        }
    }
}

/// Rutas de datos: `%APPDATA%\pngtuber-lite` o equivalente XDG/HOME.
#[derive(Clone, Debug)]
pub struct Paths {
    pub dir: PathBuf,
    pub settings: PathBuf,
    pub avatars: PathBuf,
}

impl Paths {
    pub fn discover() -> Self {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));

        let dir = base.join("pngtuber-lite");
        Self {
            settings: dir.join("settings.json"),
            avatars: dir.join("avatars"),
            dir,
        }
    }

    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(&self.avatars)?;
        Ok(())
    }

    /// Bytes de un PNG guardado, relativo al directorio de datos.
    pub fn avatar_bytes(&self, relative: &str) -> Option<Vec<u8>> {
        fs::read(self.dir.join(relative)).ok()
    }
}
