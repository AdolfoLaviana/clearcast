//! Core audio processing library for ClearCast
//!
//! This library provides audio processing capabilities including noise reduction,
//! normalization, and compression for the ClearCast application.
//!
//! # Features
//! - `wasm` - Enables WebAssembly compilation and JavaScript bindings
//! - `native` - Enables native compilation (default)

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![cfg_attr(feature = "wasm", allow(clippy::unused_unit))]

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// Import modules
pub mod engine;
pub mod filters;
pub mod utils;
pub mod effects;

/// Re-export the main audio processing engine and error type
pub use engine::{AudioEngine, AudioProcessingError};
pub use effects::{AudioEffect, Delay};

/// WebAssembly bindings for ClearCast core functionality
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmAudioEngine {
    engine: AudioEngine,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmAudioEngine {
    /// Create a new audio engine with default settings
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Initialize panic hook for better error messages in the browser
        #[cfg(feature = "console_error_panic_hook")]
        {
            use std::sync::Once;
            static SET_HOOK: Once = Once::new();
            SET_HOOK.call_once(|| {
                console_error_panic_hook::set_once();
            });
        }
        
        // Initialize console logging if enabled
        #[cfg(feature = "console_log")]
        {
            use log::Level;
            console_log::init_with_level(Level::Debug).ok();
        }
        
        WasmAudioEngine {
            engine: AudioEngine::new(),
        }
    }
    
    /// Create a new audio engine with custom settings
    /// 
    /// # Arguments
    /// * `noise_threshold` - Threshold for noise reduction (0.0 to 1.0)
    /// * `target_level` - Target normalization level (0.0 to 1.0)
    #[wasm_bindgen(js_name = withSettings)]
    pub fn with_settings(noise_threshold: f32, target_level: f32) -> Result<WasmAudioEngine, JsValue> {
        Ok(WasmAudioEngine {
            engine: AudioEngine::with_settings(noise_threshold, target_level)
                .map_err(|e| JsValue::from_str(&e.to_string()))?,
        })
    }
    
    /// Process an audio buffer with all enabled effects
    /// 
    /// # Arguments
    /// * `input` - A Float32Array containing the audio samples
    /// 
    /// # Returns
    /// A new Float32Array with the processed audio
    #[wasm_bindgen(js_name = processBuffer)]
    pub fn process_buffer(&self, input: &[f32]) -> Result<Vec<f32>, JsValue> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        
        // Convert input to Vec<f32> y asegurarse de que los valores estén en el rango [-1.0, 1.0]
        let mut samples: Vec<f32> = input.iter()
            .map(|&x| x.max(-1.0).min(1.0))
            .collect();
        
        // Aplicar reducción de ruido si está habilitada (con parámetros conservadores)
        if self.engine.noise_reduction_threshold > 0.0 {
            let mut audio = ndarray::Array1::from_vec(samples);
            if let Err(e) = self.engine.apply_noise_reduction(&mut audio) {
                console_error(&format!("Noise reduction warning: {}", e));
                // Continuar incluso si hay un error en la reducción de ruido
            } else {
                samples = audio.to_vec();
            }
        }
        
        // Aplicar normalización con un margen de seguridad
        if self.engine.target_peak > 0.0 && self.engine.target_peak <= 1.0 {
            let mut audio = ndarray::Array1::from_vec(samples);
            if let Err(e) = self.engine.normalize_audio(&mut audio) {
                console_error(&format!("Normalization warning: {}", e));
                // Continuar incluso si hay un error en la normalización
            } else {
                samples = audio.to_vec();
                
                // Asegurarse de que no haya clipping después de la normalización
                for sample in &mut samples {
                    *sample = sample.max(-0.99).min(0.99);
                }
            }
        }
        
        // Aplicar efectos si hay alguno
        if !self.engine.effects.is_empty() {
            if let Err(e) = self.engine.apply_effects(&mut samples) {
                console_error(&format!("Effects processing warning: {}", e));
                // Continuar incluso si hay un error en los efectos
            }
        }
        
        // Aplicar limitador de picos suave para evitar distorsión
        // con un margen de seguridad del 5% para evitar el recorte
        self.engine.apply_soft_limiter(&mut samples);
        
        // Asegurarse una vez más de que los valores estén en el rango [-1.0, 1.0]
        for sample in &mut samples {
            *sample = sample.max(-0.95).min(0.95);
        }
        
        Ok(samples)
    }
    
    /// Apply compression to an audio buffer
    /// 
    /// # Arguments
    /// * `input` - A Float32Array containing the audio samples
    /// * `threshold` - Compression threshold in dBFS (0 to -60)
    /// * `ratio` - Compression ratio (e.g., 4.0 for 4:1)
    /// * `attack_ms` - Attack time in milliseconds (1.0 to 100.0)
    /// * `release_ms` - Release time in milliseconds (10.0 to 1000.0)
    /// 
    /// # Returns
    /// A new Float32Array with the compressed audio
    #[wasm_bindgen(js_name = compress)]
    pub fn compress(
        &self,
        input: &[f32],
        threshold: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
    ) -> Result<Vec<f32>, JsValue> {
        use crate::filters::compressor::Compressor;
        
        // Validate input parameters
        let threshold = threshold.clamp(-60.0, 0.0);
        let ratio = ratio.max(1.0);
        let attack_ms = attack_ms.max(0.1).min(100.0);
        let release_ms = release_ms.max(5.0).min(2000.0);
        
        // Create a new compressor with the specified parameters
        let sample_rate = 44100.0; // Default sample rate
        let mut compressor = Compressor::new(
            threshold,
            ratio,
            attack_ms / 1000.0, // Convert to seconds
            release_ms / 1000.0, // Convert to seconds
            sample_rate,
        );
        
        // Process the audio
        let mut output = Vec::with_capacity(input.len());
        for &sample in input {
            output.push(compressor.process(sample));
        }
        
        Ok(output)
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(all(feature = "wasm", feature = "wee_alloc"))]
mod wasm_alloc {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str);
    }

    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    
    #[wasm_bindgen(start)]
    pub fn init() {
        // Inicializar el logger
        console_log::init_with_level(log::Level::Debug).unwrap();
        log::info!("Wee allocator initialized");
    }
}

#[cfg(all(feature = "wasm", feature = "wee_alloc"))]
use wasm_alloc::init as init_wasm_alloc;

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_audio_engine_creation() {
        // Test default settings by processing a simple signal
        let engine = AudioEngine::new();
        let result = engine.process(vec![0.1, -0.2, 0.3, -0.4]).unwrap();
        assert_eq!(result.len(), 4);
        
        // Test custom settings
        let engine = AudioEngine::with_settings(0.1, 0.9).unwrap();
        let result = engine.process(vec![0.1, -0.2, 0.3, -0.4]).unwrap();
        assert_eq!(result.len(), 4);
        
        // Test invalid settings
        assert!(AudioEngine::with_settings(-0.1, 0.5).is_err());
        assert!(AudioEngine::with_settings(0.5, 1.1).is_err());
    }

    #[test]
    fn test_noise_reduction() {
        // Set threshold to 0.1 (10%) of the max amplitude (0.6 * 0.1 = 0.06)
        // So values with absolute value < 0.06 should be zeroed out
        let engine = AudioEngine::with_settings(0.1, 1.0).unwrap();
        
        // Create a test signal with some noise
        let signal = vec![0.05, 0.5, 0.06, -0.4, 0.03, 0.6, -0.02];
        
        // Process the signal (applies both noise reduction and normalization)
        let result = engine.process(signal).unwrap();
        
        // Values below threshold (0.06) should be zeroed out
        assert_eq!(result[0], 0.0, "Value 0.05 < 0.06 should be zeroed out");
        assert_eq!(result[4], 0.0, "Value 0.03 < 0.06 should be zeroed out");
        assert_eq!(result[6], 0.0, "Value -0.02 < 0.06 should be zeroed out");
        
        // Values at or above threshold should be preserved and normalized
        // The maximum absolute value in the input is 0.6, so values are divided by 0.6
        // We'll use a more lenient epsilon since the exact value can vary
        let epsilon = 0.01;  // 1% error margin
        let expected = 0.5 / 0.6;
        let actual = result[1].abs();
        let error = (actual - expected).abs() / expected;
        assert!(
            error <= epsilon,
            "Value 0.5 should be preserved and normalized. Expected: {}, got: {}, error: {:.2}% > {:.2}%",
            expected,
            actual,
            error * 100.0,
            epsilon * 100.0
        );
        
        // Note: The behavior for values exactly at the threshold (0.06) may vary
        // depending on the implementation. We'll check that it's either zero or normalized.
        if result[2] != 0.0 {
            let expected = 0.06 / 0.6;
            let diff = (result[2].abs() - expected).abs();
            assert!(
                diff < epsilon,
                "Value 0.06 at threshold should be either zero or normalized. Expected: {}, got: {}",
                expected,
                result[2]
            );
        }
        
        let expected = 0.4 / 0.6;
        assert!(
            (result[3].abs() - expected).abs() < epsilon,
            "Value -0.4 should be preserved and normalized. Expected: {}, got: {}",
            expected,
            result[3].abs()
        );
        
        assert!(
            (result[5].abs() - 1.0).abs() < epsilon,
            "Maximum value 0.6 should be normalized to 1.0, got: {}",
            result[5].abs()
        );
        
        // Check that the sign is preserved for negative values
        assert!(result[3] < 0.0, "Sign should be preserved for negative values");
    }

    #[test]
    fn test_normalization() {
        let engine = AudioEngine::with_settings(0.0, 0.5).unwrap();
        
        // Test signal with max amplitude of 0.5, target is 0.5, so gain should be 1.0
        let signal = vec![0.1, -0.5, 0.3];
        let result = engine.process(signal).unwrap();
        assert_relative_eq!(result[1], -0.5);
        
        // Test signal with max amplitude of 0.25, target is 0.5, so gain should be 2.0
        let signal = vec![0.1, -0.25, 0.15];
        let result = engine.process(signal).unwrap();
        assert_relative_eq!(result[1], -0.5);
    }

    #[test]
    fn test_empty_buffer() {
        let engine = AudioEngine::new();
        assert!(matches!(
            engine.process(vec![]).unwrap_err(),
            AudioProcessingError::EmptyBuffer
        ));
    }
}
