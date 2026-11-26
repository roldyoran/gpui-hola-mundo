//! Enumeración de micrófonos para la UI.
//!
//! Se ejecuta bajo demanda desde el hilo de GPUI (al abrir ajustes o pulsar
//! "Actualizar"); nunca desde el callback de audio.

use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceEntry {
    /// `DeviceId` serializado con `Display` (estable entre reinicios).
    pub id: String,
    pub name: String,
}

pub fn list_input_devices() -> Result<Vec<DeviceEntry>, String> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|error| format!("No se pudieron enumerar los micrófonos: {error}"))?;

    let mut entries = Vec::new();
    for device in devices {
        let Ok(id) = device.id() else {
            continue;
        };
        entries.push(DeviceEntry {
            id: id.to_string(),
            name: device.to_string(),
        });
    }

    entries.sort_by_key(|entry| entry.name.to_lowercase());
    Ok(entries)
}
