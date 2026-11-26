//! VAD por RMS con histéresis y tiempos de retención.
//!
//! Lógica pura: no conoce cpal, ni hilos, ni GPUI. El tiempo se mide en
//! muestras (no se consulta el reloj) para que el callback de audio no haga
//! syscalls. `process` devuelve `true` solo cuando el estado cambia.

/// Tiempo mínimo por encima del umbral para abrir la boca.
pub const ATTACK_MS: u32 = 40;
/// Tiempo mínimo por debajo del umbral de cierre para cerrarla.
pub const RELEASE_MS: u32 = 250;
/// Diferencia entre el umbral de apertura y el de cierre.
pub const HYSTERESIS_DB: f32 = 6.0;

#[derive(Clone, Debug)]
pub struct Vad {
    sample_rate: u32,
    sensitivity: u8,
    open: bool,
    above: u32,
    below: u32,
}

impl Vad {
    pub fn new(sample_rate: u32, sensitivity: u8) -> Self {
        Self {
            sample_rate: sample_rate.max(1),
            sensitivity: sensitivity.min(100),
            open: false,
            above: 0,
            below: 0,
        }
    }

    /// Umbral de apertura en dBFS: 0 → −20 dB, 100 → −60 dB.
    pub fn threshold_db(sensitivity: u8) -> f32 {
        -20.0 - 40.0 * (sensitivity.min(100) as f32 / 100.0)
    }

    pub fn open_threshold_db(&self) -> f32 {
        Self::threshold_db(self.sensitivity)
    }

    pub fn close_threshold_db(&self) -> f32 {
        self.open_threshold_db() - HYSTERESIS_DB
    }

    pub fn set_sensitivity(&mut self, sensitivity: u8) {
        self.sensitivity = sensitivity.min(100);
    }

    pub fn sensitivity(&self) -> u8 {
        self.sensitivity
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Procesa un bloque de `frames` muestras a `db` dBFS.
    ///
    /// Devuelve `true` si la boca cambió de estado. `frames` es el número de
    /// tramas (muestras / canales), no el de muestras totales.
    pub fn process(&mut self, db: f32, frames: usize) -> bool {
        if frames == 0 || db.is_nan() {
            return false;
        }
        let frames = frames.min(u32::MAX as usize) as u32;

        if self.open {
            if db < self.close_threshold_db() {
                self.below = self.below.saturating_add(frames);
                if self.below >= self.release_frames() {
                    self.open = false;
                    self.below = 0;
                    self.above = 0;
                    return true;
                }
            } else {
                self.below = 0;
            }
        } else if db >= self.open_threshold_db() {
            self.above = self.above.saturating_add(frames);
            if self.above >= self.attack_frames() {
                self.open = true;
                self.above = 0;
                self.below = 0;
                return true;
            }
        } else {
            self.above = 0;
        }

        false
    }

    fn attack_frames(&self) -> u32 {
        self.frames_for(ATTACK_MS)
    }

    fn release_frames(&self) -> u32 {
        self.frames_for(RELEASE_MS)
    }

    fn frames_for(&self, ms: u32) -> u32 {
        let frames = (u64::from(self.sample_rate) * u64::from(ms)) / 1000;
        frames.clamp(1, u64::from(u32::MAX)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1 kHz: una muestra = 1 ms, así los tiempos se leen directos.
    fn vad(sensitivity: u8) -> Vad {
        Vad::new(1000, sensitivity)
    }

    #[test]
    fn umbrales_monotonos_y_en_rango() {
        assert!((Vad::threshold_db(0) + 20.0).abs() < f32::EPSILON);
        assert!((Vad::threshold_db(100) + 60.0).abs() < f32::EPSILON);
        assert!(Vad::threshold_db(30) > Vad::threshold_db(70));
        assert!(
            (vad(55).close_threshold_db() - (vad(55).open_threshold_db() - 6.0)).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn ataque_exige_el_tiempo_minimo() {
        let mut vad = vad(55);
        assert!(!vad.process(-30.0, 39));
        assert!(!vad.is_open());
        assert!(vad.process(-30.0, 1));
        assert!(vad.is_open());
    }

    #[test]
    fn release_exige_el_tiempo_minimo() {
        let mut vad = vad(55);
        assert!(vad.process(-30.0, ATTACK_MS as usize));
        assert!(vad.is_open());

        assert!(!vad.process(-55.0, (RELEASE_MS - 1) as usize));
        assert!(vad.is_open());
        assert!(vad.process(-55.0, 1));
        assert!(!vad.is_open());
    }

    #[test]
    fn histeresis_evita_parpadeo_entre_umbrales() {
        let mut vad = vad(55);
        assert!(vad.process(-30.0, ATTACK_MS as usize));
        // Entre cierre (−48) y apertura (−42): se queda abierta.
        for _ in 0..20 {
            assert!(!vad.process(-45.0, 10));
            assert!(vad.is_open());
        }
    }

    #[test]
    fn un_clic_aislado_no_abre() {
        let mut vad = vad(55);
        assert!(!vad.process(-5.0, 10));
        assert!(!vad.is_open());
        assert!(!vad.process(-70.0, 100));
        assert!(!vad.is_open());
        assert!(!vad.process(-5.0, 10));
        assert!(!vad.is_open());
    }

    #[test]
    fn voz_con_pausas_cortas_no_cierra() {
        let mut vad = vad(55);
        assert!(vad.process(-30.0, ATTACK_MS as usize));
        // Pausa de 200 ms, menor que release (250 ms).
        assert!(!vad.process(-55.0, 200));
        assert!(vad.is_open());
        // Vuelve la voz: el contador de cierre se reinicia.
        assert!(!vad.process(-30.0, 10));
        assert!(vad.is_open());
        assert!(!vad.process(-55.0, 249));
        assert!(vad.is_open());
    }

    #[test]
    fn cambio_de_sensibilidad_en_caliente() {
        let mut vad = vad(20);
        // −30 dB no abre con sensibilidad 20 (umbral −28).
        assert!(!vad.process(-30.0, ATTACK_MS as usize));
        assert!(!vad.is_open());

        vad.set_sensitivity(60); // umbral −44
        assert!(vad.process(-30.0, ATTACK_MS as usize));
        assert!(vad.is_open());

        vad.set_sensitivity(0); // umbral −20, cierre −26
        assert!(vad.process(-30.0, RELEASE_MS as usize));
        assert!(!vad.is_open());
    }

    #[test]
    fn bloques_gigantes_no_desbordan() {
        let mut vad = vad(48_000);
        assert!(vad.process(-30.0, usize::MAX));
        assert!(vad.is_open());
        assert!(vad.process(-80.0, usize::MAX));
        assert!(!vad.is_open());
    }

    #[test]
    fn bloques_vacios_o_nan_no_cambian_nada() {
        let mut vad = vad(1000);
        assert!(!vad.process(-10.0, 0));
        assert!(!vad.process(f32::NAN, 100));
        assert!(!vad.is_open());
    }
}
