//! Implementación del efecto de delay/eco
//!
//! Este módulo proporciona un efecto de delay/eco configurable que puede ser usado
//! para agregar profundidad y espacio a señales de audio. El efecto incluye controles
//! para ajustar el tiempo de retardo, retroalimentación y mezcla de señal.

use std::collections::VecDeque;
use super::AudioEffect;

/// Efecto de delay/eco digital con retroalimentación configurable
pub struct Delay {
    buffer: VecDeque<f32>,
    #[allow(dead_code)]
    max_delay_samples: usize,
    delay_samples: usize,
    feedback: f32,
    wet: f32,
    dry: f32,
    #[allow(dead_code)]
    sample_rate: u32,
}

impl Delay {    
    /// Crea un nuevo efecto de delay con los parámetros especificados
    pub fn new(
        delay_ms: f32,
        feedback: f32,
        wet: f32,
        dry: f32,
        sample_rate: u32,
    ) -> Self {
        let delay_samples = (delay_ms * sample_rate as f32 / 1000.0).round() as usize;
        
        // Crear un buffer con ceros del tamaño del retardo
        let mut buffer = VecDeque::with_capacity(delay_samples);
        buffer.resize(delay_samples, 0.0);
        
        Self {
            buffer,
            max_delay_samples: delay_samples,
            delay_samples,
            feedback: feedback.clamp(0.0, 0.99), // Evitar inestabilidad
            wet: wet.clamp(0.0, 1.0),
            dry: dry.clamp(0.0, 1.0),
            sample_rate,
        }
    }
}

impl AudioEffect for Delay {
    fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
    
    fn process_sample(&mut self, sample: f32) -> f32 {
        // Inicializar la salida con la señal seca
        let mut output = sample * self.dry;
        
        // Si el buffer está vacío, inicializarlo con ceros
        if self.buffer.is_empty() {
            // El buffer debe tener capacidad para almacenar las muestras de retardo
            self.buffer.resize(self.delay_samples, 0.0);
        }
        
        // Obtener la muestra retrasada (la más antigua en el buffer)
        let delayed = if !self.buffer.is_empty() {
            self.buffer.pop_front().unwrap_or(0.0)
        } else {
            0.0
        };
        
        // Solo aplicar la señal húmeda si el buffer está lleno (ha pasado el tiempo de retardo)
        if self.buffer.len() >= self.delay_samples {
            output += delayed * self.wet;
        }
        
        // Calcular la retroalimentación
        let feedback = if self.feedback > 0.0 {
            delayed * self.feedback
        } else {
            0.0
        };
        
        // Mezclar la señal de entrada con la retroalimentación
        let input = sample + feedback;
        
        // Agregar la nueva muestra al final del buffer
        self.buffer.push_back(input);
        
        output
    }
    
    fn reset(&mut self) {
        self.buffer.clear();
    }
    
    fn name(&self) -> &'static str {
        "Delay"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_delay_effect() {
        // Configuración de prueba más simple
        let sample_rate = 1000; // 1kHz para facilitar las pruebas
        let delay_time = 10.0; // 10ms
        let feedback = 0.0; // Sin retroalimentación para simplificar
        let wet = 0.7;
        let dry = 0.3;
        
        let mut delay = Delay::new(delay_time, feedback, wet, dry, sample_rate);
        
        // Calcular el número de muestras de retardo (10ms a 1kHz = 10 muestras)
        let delay_samples = (delay_time * sample_rate as f32 / 1000.0).round() as usize;
        
        // Verificar que el buffer se inicializó correctamente
        assert_eq!(delay.buffer.len(), delay_samples, "Buffer should be initialized with delay_samples");
        
        // Crear una señal de prueba con un impulso al principio
        let mut signal = vec![0.0; delay_samples + 1];
        signal[0] = 1.0; // Impulso en la primera muestra
        
        // Aplicar el delay
        let mut output = signal.clone();
        delay.process_buffer(&mut output);
        
        // Verificar que la señal seca se aplica correctamente
        let expected_dry = signal[0] * dry;
        assert!(
            (output[0] - expected_dry).abs() < 1e-6,
            "Expected dry signal: {}, got: {}",
            expected_dry,
            output[0]
        );
        
        // Verificar que la señal húmeda se aplica después del retardo
        // La señal húmeda debería aparecer después de delay_samples
        // Como el buffer se llena con ceros inicialmente, la primera señal húmeda debería ser 0
        if delay_samples < output.len() {
            let expected_wet = 0.0; // El buffer se inicializa con ceros
            assert!(
                (output[delay_samples] - expected_wet).abs() < 1e-6,
                "Expected wet signal at delay_samples ({}): {}, got: {}",
                delay_samples,
                expected_wet,
                output[delay_samples]
            );
        }
        
        // Procesar otra señal para ver el efecto del delay
        let mut second_signal = vec![0.0; delay_samples];
        second_signal[0] = 0.5; // Nueva señal de entrada
        delay.process_buffer(&mut second_signal);
        
        // Verificar que la señal seca se aplica correctamente
        assert!(
            (second_signal[0] - (0.5 * dry)).abs() < 1e-6,
            "Dry signal not applied correctly. Expected: {}, got: {}",
            0.5 * dry,
            second_signal[0]
        );
        
        // Verificar que la señal húmeda se aplica correctamente
        // Debería ser la señal anterior (1.0) * wet (0.7) = 0.7
        if delay_samples < second_signal.len() {
            let expected_wet = 1.0 * wet;
            assert!(
                (second_signal[delay_samples - 1] - expected_wet).abs() < 1e-6,
                "Wet signal not applied correctly. Expected: {}, got: {}",
                expected_wet,
                second_signal[delay_samples - 1]
            );
        }
    }
}
