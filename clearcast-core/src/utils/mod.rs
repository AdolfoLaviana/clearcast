//! Utility functions for ClearCast

use std::f32::consts::PI;

/// Converts frequency in Hz to angular frequency (radians/sample)
pub fn hz_to_radians(frequency: f32, sample_rate: f32) -> f32 {
    2.0 * PI * frequency / sample_rate
}

/// Normalizes a vector of audio samples to the range [-1.0, 1.0]
pub fn normalize_audio(samples: &mut [f32]) {
    if samples.is_empty() {
        return;
    }

    // Find the maximum absolute value
    let max_val = samples
        .iter()
        .fold(0.0f32, |max, &x| max.max(x.abs()));

    // Avoid division by zero
    if max_val > 0.0 {
        let scale = 1.0 / max_val;
        for sample in samples.iter_mut() {
            *sample *= scale;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_hz_to_radians() {
        let result = hz_to_radians(1000.0, 44100.0);
        assert!(result > 0.142 && result < 0.143); // ~0.1425
    }

    #[wasm_bindgen_test]
    fn test_normalize_audio() {
        let mut samples = vec![0.5, 1.0, -0.5];
        normalize_audio(&mut samples);
        assert_eq!(samples, [0.5, 1.0, -0.5]);

        let mut samples = vec![0.25, 0.5, -0.25];
        normalize_audio(&mut samples);
        assert_eq!(samples, [0.5, 1.0, -0.5]);
    }
}
