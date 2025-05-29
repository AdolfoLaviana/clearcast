//! ClearCastProcessor integrates multiple audio processing effects into a single pipeline.
//! The processing is applied in the following order:
//! 1. Wiener filter for noise reduction
//! 2. Parametric EQ for frequency shaping
//! 3. Multiband compression
//! 4. Soft limiting to prevent clipping
//! 5. RMS normalization

use crate::filters::{
    compressor::compress_rms,
    equalizer::parametric_eq,
    wiener_filter::reduce_noise_wiener,
};
use ndarray::Array1;

/// Main processor that combines multiple audio effects
pub struct ClearCastProcessor {
    sample_rate: f32,
    noise_profile: Vec<f32>,
    eq_bands: (f32, f32, f32), // (low, mid, high) gains in dB
    compressor_params: (f32, f32, f32, f32), // (threshold, ratio, attack, release)
    target_rms: f32,
    limiter_threshold: f32,
}

impl ClearCastProcessor {
    /// Creates a new ClearCastProcessor with default settings
    /// 
    /// # Arguments
    /// * `sample_rate` - The sample rate of the audio in Hz
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            noise_profile: vec![0.01; 1024], // Default noise profile
            eq_bands: (0.0, 0.0, 0.0),      // Flat EQ by default
            compressor_params: (-20.0, 4.0, 10.0, 100.0), // threshold, ratio, attack, release
            target_rms: 0.1,                // Target RMS level (0.0 to 1.0)
            limiter_threshold: 0.95,        // Limiter threshold (0.0 to 1.0)
        }
    }


    /// Configures the noise reduction parameters
    /// 
    /// # Arguments
    /// * `noise_profile` - The noise profile to use for the Wiener filter
    /// * `fft_size` - Size of the FFT window (must be a power of 2)
    /// * `hop_size` - Hop size between windows
    /// * `smoothing` - Smoothing factor (0.0 to 1.0)
    pub fn configure_noise_reduction(
        &mut self,
        noise_profile: Vec<f32>,
        fft_size: usize,
        hop_size: usize,
        smoothing: f32,
    ) {
        self.noise_profile = noise_profile;
        // Store these parameters for later use
        self.noise_profile.push(fft_size as f32);
        self.noise_profile.push(hop_size as f32);
        self.noise_profile.push(smoothing);
    }

    /// Configures the parametric EQ
    /// 
    /// # Arguments
    /// * `low_gain` - Gain for low frequencies (<200 Hz) in dB
    /// * `mid_gain` - Gain for mid frequencies (200-3000 Hz) in dB
    /// * `high_gain` - Gain for high frequencies (>3000 Hz) in dB
    pub fn configure_eq(&mut self, low_gain: f32, mid_gain: f32, high_gain: f32) {
        self.eq_bands = (low_gain, mid_gain, high_gain);
    }

    /// Configures the multiband compressor
    /// 
    /// # Arguments
    /// * `threshold` - Threshold in dBFS where compression begins
    /// * `ratio` - Compression ratio (e.g., 4.0 for 4:1)
    /// * `attack_ms` - Attack time in milliseconds
    /// * `release_ms` - Release time in milliseconds
    pub fn configure_compressor(&mut self, threshold: f32, ratio: f32, attack_ms: f32, release_ms: f32) {
        self.compressor_params = (threshold, ratio, attack_ms, release_ms);
    }

    /// Sets the target RMS level for normalization
    /// 
    /// # Arguments
    /// * `target_rms` - Target RMS level (0.0 to 1.0)
    pub fn set_target_rms(&mut self, target_rms: f32) {
        self.target_rms = target_rms.max(0.0).min(1.0);
    }

    /// Sets the limiter threshold
    /// 
    /// # Arguments
    /// * `threshold` - Limiter threshold (0.0 to 1.0)
    pub fn set_limiter_threshold(&mut self, threshold: f32) {
        self.limiter_threshold = threshold.max(0.0).min(1.0);
    }

    /// Applies soft limiting to prevent clipping
    fn apply_soft_limiter(&self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            // Simple soft clipping algorithm
            let abs_sample = sample.abs();
            if abs_sample > self.limiter_threshold {
                // Apply a smooth curve that approaches 1.0
                *sample = sample.signum() * 
                    (self.limiter_threshold + (1.0 - (-(abs_sample - self.limiter_threshold) * 10.0).exp()));
            }
        }
    }

    /// Normalizes the audio to the target RMS level
    fn normalize_rms(&self, samples: &mut [f32]) {
        // Calculate current RMS
        let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
        let rms = (sum_sq / samples.len() as f32).sqrt();
        
        // Avoid division by zero
        if rms < f32::EPSILON {
            return;
        }
        
        // Calculate scaling factor
        let scale = self.target_rms / rms;
        
        // Apply scaling
        for sample in samples.iter_mut() {
            *sample *= scale;
        }
    }

    /// Processes an audio buffer through the entire processing chain
    /// 
    /// # Arguments
    /// * `input` - Input audio buffer
    /// 
    /// # Returns
    /// Processed audio buffer
    pub fn process_audio(&mut self, input: &[f32]) -> Vec<f32> {
        if input.is_empty() {
            return Vec::new();
        }

        // 1. Apply noise reduction (Wiener filter)
        let mut processed = if self.noise_profile.len() > 3 {
            let fft_size = self.noise_profile[self.noise_profile.len() - 3] as usize;
            let hop_size = self.noise_profile[self.noise_profile.len() - 2] as usize;
            let smoothing = self.noise_profile[self.noise_profile.len() - 1];
            let noise_profile = &self.noise_profile[..self.noise_profile.len() - 3];
            
            if !noise_profile.is_empty() {
                reduce_noise_wiener(input, noise_profile, fft_size, hop_size, smoothing)
            } else {
                input.to_vec()
            }
        } else {
            input.to_vec()
        };

        // 2. Apply parametric EQ
        if self.eq_bands != (0.0, 0.0, 0.0) {
            processed = parametric_eq(
                &processed,
                self.sample_rate,
                self.eq_bands.0,
                self.eq_bands.1,
                self.eq_bands.2,
            );
        }

        // 3. Apply compression
        processed = compress_rms(
            &processed,
            self.compressor_params.0, // threshold
            self.compressor_params.1, // ratio
            self.compressor_params.2, // attack
            self.compressor_params.3, // release
            self.sample_rate,
        );

        // 4. Apply soft limiter
        self.apply_soft_limiter(&mut processed);

        // 5. Normalize to target RMS
        self.normalize_rms(&mut processed);

        processed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_processor_chain() {
        // Create a test signal (sine wave)
        let sample_rate = 44100.0;
        let freq = 1000.0;
        let duration = 0.1; // 100ms
        let num_samples = (sample_rate * duration) as usize;
        let mut signal = Vec::with_capacity(num_samples);
        
        for i in 0..num_samples {
            let t = i as f32 / sample_rate;
            signal.push((2.0 * std::f32::consts::PI * freq * t).sin() * 0.5);
        }

        // Create and configure the processor
        let mut processor = ClearCastProcessor::new(sample_rate);
        
        // Configure with some settings
        processor.configure_eq(2.0, 0.0, -1.0); // Slight bass boost, slight treble cut
        processor.configure_compressor(-20.0, 4.0, 10.0, 100.0);
        processor.set_target_rms(0.1);
        processor.set_limiter_threshold(0.9);

        // Process the signal
        let processed = processor.process_audio(&signal);

        // Basic validation
        assert_eq!(processed.len(), signal.len());
        
        // Check that the output is not all zeros
        let max_val = processed.iter().fold(0.0f32, |max, &x| max.max(x.abs()));
        assert!(max_val > 0.0, "Output should not be silent");
        
        // Check that the output is within bounds
        for &sample in &processed {
            assert!(sample >= -1.0 && sample <= 1.0, "Sample out of range: {}", sample);
        }
    }

    #[test]
    fn test_empty_input() {
        let mut processor = ClearCastProcessor::new(44100.0);
        let result = processor.process_audio(&[]);
        assert!(result.is_empty());
    }
}
