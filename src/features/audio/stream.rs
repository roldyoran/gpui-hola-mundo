//! Construcción del stream de captura y cálculo de RMS.
//!
//! Este módulo se ejecuta en el hilo dueño del audio, nunca en la UI. El
//! callback no asigna memoria, no bloquea y no consulta el reloj: solo suma
//! cuadrados, aplica el VAD y publica un evento cuando la boca cambia.

use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU32, Ordering};

use async_channel::Sender;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, StreamConfig};

use super::vad::Vad;
use super::{AudioConfig, AudioEvent};

pub struct ActiveStream {
    /// Se conserva para mantener vivo el stream; nunca se lee.
    _stream: cpal::Stream,
    pub device_name: String,
}

impl ActiveStream {
    fn play(&self) -> Result<(), String> {
        self._stream
            .play()
            .map_err(|error| format!("No se pudo iniciar la captura: {error}"))
    }
}

/// Abre el dispositivo configurado (o el predeterminado) y arranca la captura.
pub fn open(
    config: &AudioConfig,
    sensitivity: Arc<AtomicU8>,
    level: Arc<AtomicU32>,
    events: Sender<AudioEvent>,
) -> Result<ActiveStream, String> {
    let host = cpal::default_host();
    let device = config
        .device_id
        .as_deref()
        .and_then(|id| cpal::DeviceId::from_str(id).ok())
        .and_then(|id| host.device_by_id(&id))
        .or_else(|| host.default_input_device())
        .ok_or_else(|| "No hay micrófono disponible".to_string())?;

    let device_name = device.to_string();
    let supported = device
        .default_input_config()
        .map_err(|error| format!("El micrófono no ofrece entrada: {error}"))?;
    let sample_format = supported.sample_format();
    let stream_config = supported.config();
    let channels = usize::from(stream_config.channels).max(1);
    let sample_rate = stream_config.sample_rate;

    let stream = match sample_format {
        SampleFormat::F32 => build::<f32>(
            &device,
            stream_config,
            channels,
            sample_rate,
            sensitivity,
            level,
            events,
        ),
        SampleFormat::F64 => build::<f64>(
            &device,
            stream_config,
            channels,
            sample_rate,
            sensitivity,
            level,
            events,
        ),
        SampleFormat::I16 => build::<i16>(
            &device,
            stream_config,
            channels,
            sample_rate,
            sensitivity,
            level,
            events,
        ),
        SampleFormat::I32 => build::<i32>(
            &device,
            stream_config,
            channels,
            sample_rate,
            sensitivity,
            level,
            events,
        ),
        SampleFormat::I8 => build::<i8>(
            &device,
            stream_config,
            channels,
            sample_rate,
            sensitivity,
            level,
            events,
        ),
        SampleFormat::U16 => build::<u16>(
            &device,
            stream_config,
            channels,
            sample_rate,
            sensitivity,
            level,
            events,
        ),
        SampleFormat::U8 => build::<u8>(
            &device,
            stream_config,
            channels,
            sample_rate,
            sensitivity,
            level,
            events,
        ),
        other => Err(format!("Formato de audio no soportado: {other:?}")),
    }?;

    let active = ActiveStream {
        _stream: stream,
        device_name,
    };
    active.play()?;
    Ok(active)
}

fn build<T>(
    device: &cpal::Device,
    config: StreamConfig,
    channels: usize,
    sample_rate: u32,
    sensitivity: Arc<AtomicU8>,
    level: Arc<AtomicU32>,
    events: Sender<AudioEvent>,
) -> Result<cpal::Stream, String>
where
    T: SizedSample + Send + 'static,
    f32: FromSample<T>,
{
    let mut vad = Vad::new(sample_rate, sensitivity.load(Ordering::Relaxed));
    let error_events = events.clone();

    device
        .build_input_stream::<T, _, _>(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                if data.is_empty() {
                    return;
                }
                vad.set_sensitivity(sensitivity.load(Ordering::Relaxed));

                let mut sum = 0.0_f64;
                for sample in data {
                    let value = f64::from(f32::from_sample(*sample));
                    sum += value * value;
                }
                let rms = (sum / data.len() as f64).sqrt();
                let db = (20.0 * rms.max(1e-7).log10()) as f32;
                let frames = data.len() / channels;

                level.store(db.to_bits(), Ordering::Relaxed);

                if vad.process(db, frames) {
                    let _ = events.try_send(AudioEvent::Mouth(vad.is_open()));
                }
            },
            move |error| {
                let _ = error_events.try_send(AudioEvent::Error(error.to_string()));
            },
            None,
        )
        .map_err(|error| format!("No se pudo abrir el micrófono: {error}"))
}
