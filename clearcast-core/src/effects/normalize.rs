//! Módulo para normalización de audio basada en RMS
//!
//! Este módulo proporciona funciones para normalizar el nivel de audio utilizando
//! el valor RMS (Root Mean Square) como referencia. La normalización ajusta la
//! ganancia de la señal para que su nivel RMS coincida con un valor objetivo
//! especificado en dBFS (decibelios relativos a la escala completa).

use std::f32::consts::SQRT_2;

/// Normaliza un búfer de audio al nivel RMS objetivo especificado en dBFS.
///
/// # Argumentos
///
/// * `buffer` - Búfer de audio a normalizar (modificado in-place)
/// * `target_dbfs` - Nivel objetivo en dBFS (valores negativos, ej: -16.0 para -16 dBFS)
///
/// # Ejemplo
///
/// ```no_run
/// let mut audio_buffer = vec![0.1, -0.2, 0.15, -0.05, 0.3];
/// clearcast_core::effects::normalize_rms(&mut audio_buffer, -12.0);
/// ```
pub fn normalize_rms(buffer: &mut [f32], target_dbfs: f32) {
    if buffer.is_empty() {
        return;
    }

    // Calcular el valor RMS actual
    let sum_squares: f32 = buffer.iter().map(|&x| x * x).sum();
    let rms = (sum_squares / buffer.len() as f32).sqrt();
    
    // Evitar división por cero si el audio es silencio
    if rms <= f32::MIN_POSITIVE {
        return;
    }
    
    // Convertir el objetivo de dBFS a amplitud lineal
    let target_linear = 10.0f32.powf(target_dbfs / 20.0);
    
    // Calcular el factor de escala necesario
    let scale_factor = target_linear / rms;
    
    // Aplicar el factor de escala a todas las muestras
    for sample in buffer.iter_mut() {
        *sample *= scale_factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_normalize_rms_silent() {
        let mut silent = vec![0.0; 100];
        let original = silent.clone();
        normalize_rms(&mut silent, -12.0);
        assert_eq!(silent, original, "El búfer silencioso no debería cambiar");
    }
    
    #[test]
    fn test_normalize_rms_simple() {
        // Señal de prueba con RMS de 0.5 (asumiendo muestras [1.0, -1.0, 1.0, -1.0])
        let mut signal = vec![1.0, -1.0, 1.0, -1.0];
        let target_dbfs = -12.0;
        let target_linear = 10.0f32.powf(target_dbfs / 20.0);
        
        normalize_rms(&mut signal, target_dbfs);
        
        // Calcular RMS después de la normalización
        let sum_squares: f32 = signal.iter().map(|&x| x * x).sum();
        let rms_after = (sum_squares / signal.len() as f32).sqrt();
        
        // Verificar que el RMS después de la normalización esté cerca del objetivo
        assert_relative_eq!(rms_after, target_linear, epsilon = 1e-6);
        
        // Verificar que la forma de onda se mantuvo
        assert_relative_eq!(signal[0], -signal[1], epsilon = 1e-6);
        assert_relative_eq!(signal[0], signal[2], epsilon = 1e-6);
    }
    
    #[test]
    fn test_normalize_rms_already_at_target() {
        // Crear una señal con RMS de -12 dBFS
        let target_dbfs = -12.0;
        let target_linear = 10.0f32.powf(target_dbfs / 20.0);
        let mut signal = vec![target_linear, -target_linear, target_linear, -target_linear];
        let expected = signal.clone();
        
        normalize_rms(&mut signal, target_dbfs);
        
        // La señal no debería cambiar ya que ya está en el nivel objetivo
        assert_relative_eq!(signal.as_slice(), expected.as_slice(), epsilon = 1e-6);
    }
}
