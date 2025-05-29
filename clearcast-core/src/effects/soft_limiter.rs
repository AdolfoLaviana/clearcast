//! Módulo para el efecto de limitador suave (Soft Limiter)
//!
//! Este módulo proporciona un limitador de audio que aplica una función de transferencia
//! suave basada en tangente hiperbólica (tanh) para prevenir recortes (clipping) en la señal de audio.
//! A diferencia de un limitador duro, este efecto proporciona una transición más suave al límite,
//! lo que resulta en una distorsión menos perceptible.

use crate::effects::AudioEffect;
use std::f32::consts::{E, PI};

/// Un limitador suave que aplica una función de transferencia basada en tanh
///
/// Este efecto es útil para prevenir picos de amplitud sin introducir distorsión dura.
/// La función de transferencia se comporta de manera lineal para señales de bajo nivel
/// y se suaviza gradualmente a medida que la señal se acerca al límite.
#[derive(Debug, Clone)]
pub struct SoftLimiter {
    /// Nivel máximo de salida (normalmente entre 0.5 y 1.0)
    threshold: f32,
    /// Factor de suavizado (controla la transición a la región de limitación)
    knee: f32,
    /// Si es true, el limitador está activado
    is_active: bool,
}

impl SoftLimiter {
    /// Crea un nuevo limitador suave con el umbral especificado
    ///
    /// # Argumentos
    /// * `threshold` - Nivel de umbral (0.0 a 1.0) donde comienza la limitación
    /// * `knee` - Ancho de la rodilla (0.0 a 1.0) que controla la suavidad de la transición
    ///
    /// # Ejemplo
    /// ```
    /// use clearcast_core::effects::SoftLimiter;
    ///
    /// let mut limiter = SoftLimiter::new(0.8, 0.1);
    /// ```
    pub fn new(threshold: f32, knee: f32) -> Self {
        Self {
            threshold: threshold.clamp(0.01, 1.0),
            knee: knee.clamp(0.0, 1.0),
            is_active: true,
        }
    }

    /// Establece el umbral del limitador
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.01, 1.0);
    }

    /// Establece el ancho de la rodilla
    pub fn set_knee(&mut self, knee: f32) {
        self.knee = knee.clamp(0.0, 1.0);
    }

    /// Habilita o deshabilita el limitador
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    /// Aplica la función de transferencia del limitador a un valor de muestra
    fn apply_limiter(&self, sample: f32) -> f32 {
        if !self.is_active {
            return sample;
        }

        // Aplicar la función de transferencia basada en tanh
        let sign = sample.signum();
        let abs_sample = sample.abs();
        
        // Si la muestra está por debajo del umbral, devolver sin cambios
        if abs_sample <= self.threshold {
            return sample;
        }
        
        // Calcular la cantidad que excede el umbral
        let over = abs_sample - self.threshold;
        
        // Aplicar una función de transferencia suave basada en tanh
        // La función es aproximadamente lineal cerca de cero y se aplana suavemente
        let soft_limit = self.threshold + (self.knee * (over / self.knee).tanh());
        
        // Asegurarse de que no exceda 1.0
        let limited = sign * soft_limit.min(1.0);
        
        // Mezclar entre la señal original y la limitada para una transición más suave
        // Usar una mezcla basada en cuánto excede el umbral
        let mix = ((abs_sample - self.threshold) / (1.0 - self.threshold)).min(1.0);
        limited * mix + sample * (1.0 - mix)
    }
}

impl AudioEffect for SoftLimiter {
    /// Procesa una sola muestra de audio a través del limitador
    fn process_sample(&mut self, sample: f32) -> f32 {
        self.apply_limiter(sample)
    }

    /// Procesa un búfer completo de audio
    fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.apply_limiter(*sample);
        }
    }

    /// Reinicia el estado interno del limitador (si lo tuviera)
    fn reset(&mut self) {
        // Este limitador no tiene estado interno que reiniciar
    }

    /// Devuelve el nombre del efecto
    fn name(&self) -> &'static str {
        "SoftLimiter"
    }
}

/// Función de conveniencia para aplicar un limitador suave a un slice de audio
///
/// Esta función es útil para procesar audio sin necesidad de crear una instancia del limitador.
///
/// # Argumentos
/// * `input` - Slice de muestras de audio de entrada
/// * `threshold` - Nivel de umbral (0.0 a 1.0) donde comienza la limitación
/// * `knee` - Ancho de la rodilla (0.0 a 1.0) que controla la suavidad de la transición
///
/// # Ejemplo
/// ```
/// use clearcast_core::effects::soft_limiter::soft_limit_buffer;
///
/// let mut audio = vec![0.5, 1.5, -1.8, 0.3];
/// soft_limit_buffer(&audio, &mut audio, 0.8, 0.05);
/// ```
pub fn soft_limit_buffer(input: &[f32], output: &mut [f32], threshold: f32, knee: f32) {
    let mut limiter = SoftLimiter::new(threshold, knee);
    for (i, &sample) in input.iter().enumerate() {
        output[i] = limiter.process_sample(sample);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;

    #[test]
    fn test_soft_limiter_below_threshold() {
        let limiter = SoftLimiter::new(0.8, 0.1);
        let sample = 0.5;
        assert_eq!(limiter.apply_limiter(sample), sample);
    }

    #[test]
    fn test_soft_limiter_above_threshold() {
        let limiter = SoftLimiter::new(0.8, 0.1);
        let sample = 1.5;
        let result = limiter.apply_limiter(sample);
        assert!(result < sample);
        assert!(result > 0.8);
    }

    #[test]
    fn test_soft_limiter_negative() {
        let limiter = SoftLimiter::new(0.8, 0.1);
        let sample = -1.5;
        let result = limiter.apply_limiter(sample);
        assert!(result > sample);
        assert!(result < -0.8);
    }

    #[test]
    fn test_soft_limiter_buffer() {
        let mut limiter = SoftLimiter::new(0.8, 0.1);
        let input = [0.5, 1.5, -1.8, 0.3];
        let mut output = [0.0; 4];
        
        for i in 0..input.len() {
            output[i] = limiter.process_sample(input[i]);
        }
        
        assert_eq!(output[0], 0.5);  // Por debajo del umbral
        assert!(output[1] < 1.5 && output[1] > 0.8);  // Por encima del umbral
        assert!(output[2] > -1.8 && output[2] < -0.8); // Por debajo del umbral negativo
        assert_eq!(output[3], 0.3);  // Por debajo del umbral
    }

    #[test]
    fn test_soft_limit_buffer_function() {
        let input = [0.5, 1.5, -1.8, 0.3];
        let mut output = [0.0; 4];
        
        soft_limit_buffer(&input, &mut output, 0.8, 0.1);
        
        assert_eq!(output[0], 0.5);  // Por debajo del umbral
        assert!(output[1] < 1.5 && output[1] > 0.8);  // Por encima del umbral
        assert!(output[2] > -1.8 && output[2] < -0.8); // Por debajo del umbral negativo
        assert_eq!(output[3], 0.3);  // Por debajo del umbral
    }
}
