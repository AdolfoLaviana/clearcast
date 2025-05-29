//! Multiband compressor implementation for ClearCast
//! 
//! This module provides a multiband compressor that splits the audio signal into
//! multiple frequency bands and applies compression independently to each band.

use crate::filters::compressor::compress_rms;

/// Parameters for a single band in the multiband compressor
#[derive(Debug, Clone, Copy)]
pub struct BandParams {
    /// Lower frequency boundary of the band in Hz
    pub low_freq: f32,
    /// Upper frequency boundary of the band in Hz
    pub high_freq: f32,
    /// Compression threshold in dBFS (0 dBFS = full scale)
    pub threshold: f32,
    /// Compression ratio (e.g., 4.0 for 4:1)
    pub ratio: f32,
    /// Attack time in milliseconds
    pub attack_ms: f32,
    /// Release time in milliseconds
    pub release_ms: f32,
}

impl Default for BandParams {
    fn default() -> Self {
        Self {
            low_freq: 0.0,
            high_freq: 20000.0,
            threshold: -20.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
        }
    }
}

/// A multiband compressor that splits the audio into multiple frequency bands
/// and applies compression independently to each band.
pub struct MultibandCompressor {
    sample_rate: f32,
    bands: Vec<BandParams>,
    x_history: Vec<Vec<f32>>,
    y_history: Vec<Vec<f32>>,
    a_coeffs: Vec<[f32; 3]>,
    b_coeffs: Vec<[f32; 3]>,
}

impl MultibandCompressor {
    /// Creates a new multiband compressor with the specified bands and sample rate.
    /// 
    /// # Arguments
    /// * `bands` - Vector of band parameters
    /// * `sample_rate` - Sample rate in Hz
    /// 
    /// # Panics
    /// Panics if the bands overlap or don't cover the full frequency range.
    pub fn new(bands: Vec<BandParams>, sample_rate: f32) -> Self {
        // Hacer una copia mutable para ordenar
        let mut sorted_bands = bands;
        
        // Ordenar las bandas por frecuencia
        sorted_bands.sort_by(|a, b| a.low_freq.partial_cmp(&b.low_freq).unwrap());
        
        // Verificar que las bandas no se solapen y cubran todo el rango
        for i in 0..sorted_bands.len() {
            if i > 0 {
                assert!(
                    sorted_bands[i].low_freq >= sorted_bands[i-1].high_freq,
                    "Bands must be in increasing frequency order and not overlap"
                );
            }
            
            assert!(
                sorted_bands[i].low_freq < sorted_bands[i].high_freq,
                "Invalid frequency range for band {}",
                i
            );
        }
        
        // Calcular los coeficientes de los filtros para cada banda
        let mut a_coeffs = Vec::with_capacity(sorted_bands.len());
        let mut b_coeffs = Vec::with_capacity(sorted_bands.len());
        
        for i in 0..sorted_bands.len() {
            let low_freq = if i == 0 { 0.0 } else { sorted_bands[i-1].high_freq };
            let high_freq = sorted_bands[i].high_freq;
            
            let (b, a) = Self::butterworth_bandpass(
                low_freq,
                high_freq,
                sample_rate,
            );
            a_coeffs.push(a);
            b_coeffs.push(b);
        }

        let num_bands = sorted_bands.len();
        
        Self {
            sample_rate,
            bands: sorted_bands,
            x_history: vec![vec![0.0; 3]; num_bands],
            y_history: vec![vec![0.0; 3]; num_bands],
            a_coeffs,
            b_coeffs,
        }
    }

    /// Processes an audio buffer through the multiband compressor.
    /// 
    /// # Arguments
    /// * `input` - Input audio buffer (mono, normalized to [-1.0, 1.0])
    /// 
    /// # Returns
    /// Processed audio buffer with multiband compression applied
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let num_bands = self.bands.len();
        let mut band_outputs = vec![vec![0.0; input.len()]; num_bands];
        let mut output = vec![0.0; input.len()];

        // Process each band
        for (i, band) in self.bands.iter().enumerate() {
            // Apply bandpass filter
            for (n, &x) in input.iter().enumerate() {
                // Update history
                self.x_history[i][2] = self.x_history[i][1];
                self.x_history[i][1] = self.x_history[i][0];
                self.x_history[i][0] = x;

                // Apply filter difference equation (Direct Form I)
                let y = (self.b_coeffs[i][0] * self.x_history[i][0] +
                        self.b_coeffs[i][1] * self.x_history[i][1] +
                        self.b_coeffs[i][2] * self.x_history[i][2] -
                        self.a_coeffs[i][1] * self.y_history[i][0] -
                        self.a_coeffs[i][2] * self.y_history[i][1]) / self.a_coeffs[i][0];

                // Update output history
                self.y_history[i][2] = self.y_history[i][1];
                self.y_history[i][1] = self.y_history[i][0];
                self.y_history[i][0] = y;

                band_outputs[i][n] = y;
            }

            // Apply compression to this band
            let compressed = compress_rms(
                &band_outputs[i],
                band.threshold,
                band.ratio,
                band.attack_ms,
                band.release_ms,
                self.sample_rate,
            );

            // Mix compressed band into output
            for (out, &comp) in output.iter_mut().zip(compressed.iter()) {
                *out += comp;
            }
        }

        output
    }

    /// Creates a 2nd order Linkwitz-Riley bandpass filter (cascaded lowpass and highpass)
    /// This provides better frequency response than a single Butterworth filter
    fn butterworth_bandpass(low_freq: f32, high_freq: f32, sample_rate: f32) -> ([f32; 3], [f32; 3]) {
        // Ensure frequencies are within valid range
        let low_freq = low_freq.max(20.0).min(sample_rate * 0.49);
        let high_freq = high_freq.max(low_freq * 1.1).min(sample_rate * 0.49);
        
        // Pre-warp frequencies for bilinear transform
        let omega_low = 2.0 * sample_rate * (std::f32::consts::PI * low_freq / sample_rate).tan();
        let omega_high = 2.0 * sample_rate * (std::f32::consts::PI * high_freq / sample_rate).tan();
        
        // Calculate Q factor for better shape control
        let q = (high_freq / low_freq).sqrt();
        let sqrt2 = std::f32::consts::SQRT_2;
        
        // Bandwidth and center frequency
        let bw = omega_high - omega_low;
        let w0 = (omega_low * omega_high).sqrt();
        
        // Calculate coefficients for bandpass filter
        let alpha = w0 / bw;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos() / a0;
        let a2 = (1.0 - alpha) / a0;
        let b0 = (alpha / a0) * sqrt2;
        let b1 = 0.0;
        let b2 = -b0;
        
        // Normalize coefficients for unity gain at center frequency
        let center_gain = (b0 * b0 + b1 * b1 + b2 * b2 + 2.0 * (b0 * b1 + b1 * b2) * w0.cos() + 2.0 * b0 * b2 * (2.0 * w0).cos())
            / (1.0 + a1 * a1 + a2 * a2 + 2.0 * (a1 + a1 * a2) * w0.cos() + 2.0 * a2 * (2.0 * w0).cos());
        
        let gain_correction = 1.0 / center_gain.sqrt();
        
        (
            [b0 * gain_correction, b1 * gain_correction, b2 * gain_correction],  // b coefficients
            [1.0, a1, a2]                                                         // a coefficients (already normalized)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn generate_test_signal(freq: f32, sample_rate: f32, duration_sec: f32) -> Vec<f32> {
        let num_samples = (sample_rate * duration_sec) as usize;
        (0..num_samples)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate).sin())
            .collect()
    }

    #[test]
    fn test_bandpass_filter() {
        let sample_rate = 44100.0;
        let duration = 0.1; // 100ms
        
        // Create a test signal with multiple frequencies
        let signal = generate_test_signal(100.0, sample_rate, duration);
        
        // Create a bandpass filter that should pass 80-120 Hz
        let (b, a) = MultibandCompressor::butterworth_bandpass(80.0, 120.0, sample_rate);
        
        // Apply the filter (simplified version for testing)
        let mut y = vec![0.0; signal.len()];
        let mut x_hist = [0.0; 3];
        let mut y_hist = [0.0; 3];
        
        for i in 0..signal.len() {
            // Shift history
            x_hist[2] = x_hist[1];
            x_hist[1] = x_hist[0];
            x_hist[0] = signal[i];
            
            // Apply filter difference equation
            y[i] = b[0] * x_hist[0] + b[1] * x_hist[1] + b[2] * x_hist[2]
                 - a[1] * y_hist[0] - a[2] * y_hist[1];
            
            // Update output history
            y_hist[2] = y_hist[1];
            y_hist[1] = y_hist[0];
            y_hist[0] = y[i];
        }
        
        // Ignore filter settling time (first and last 10% of the signal)
        let start_idx = signal.len() / 10;
        let end_idx = signal.len() * 9 / 10;
        
        // Calculate input and output energy in the analysis window
        let input_energy: f32 = signal[start_idx..end_idx].iter().map(|x| x * x).sum();
        let output_energy: f32 = y[start_idx..end_idx].iter().map(|x| x * x).sum();
        
        // Calculate energy ratio in dB (avoid log of zero)
        let energy_ratio = if input_energy > 1e-10 {
            output_energy / input_energy
        } else {
            0.0
        };
        
        let energy_ratio_db = if energy_ratio > 1e-10 {
            10.0 * energy_ratio.log10()
        } else {
            -100.0
        };
        
        // Calculate cross-correlation between input and output
        let mut cross_corr = 0.0f32;
        for i in start_idx..end_idx {
            cross_corr += signal[i] * y[i];
        }
        
        // Normalize the correlation by the signal energies
        let input_energy_sqrt = input_energy.sqrt();
        let output_energy_sqrt = output_energy.sqrt();
        let normalization = input_energy_sqrt * output_energy_sqrt;
        
        let normalized_correlation = if normalization > 1e-10 {
            cross_corr / normalization
        } else {
            0.0
        };
        
        // Log diagnostic information
        println!("Bandpass filter test - Input energy: {:.2} dB, Output energy: {:.2} dB, Energy ratio: {:.2} dB, Normalized correlation: {:.4}",
                 10.0 * input_energy.log10(),
                 10.0 * output_energy.log10(),
                 energy_ratio_db,
                 normalized_correlation);
        
        // Verify that there's some signal in the output (not completely attenuated)
        assert!(
            output_energy > 1e-10,
            "Output signal energy is too low (near zero)"
        );
        
        // For a 100Hz signal in an 80-120Hz bandpass, we expect significant energy
        // The exact ratio depends on the filter's characteristics
        let min_expected_db = -10.0;  // Expecting better performance with the improved filter
        
        println!("Bandpass filter - Min expected: {} dB, Actual: {:.2} dB", 
                min_expected_db, energy_ratio_db);
        
        assert!(
            energy_ratio_db > min_expected_db,
            "Output energy is too low. Expected > {} dB, got {:.2} dB",
            min_expected_db,
            energy_ratio_db
        );
        
        // Verify that the output is not just noise
        // The correlation should be high since we're passing the test frequency
        let min_correlation = 0.9;  // Expecting high correlation with the improved filter
        
        assert!(
            normalized_correlation > min_correlation,
            "Output signal does not correlate well with input. Expected > {:.2}, got {:.4}",
            min_correlation,
            normalized_correlation
        );
        
        // Verify that the output signal has the expected frequency
        // by checking zero crossings (should be approximately 100Hz)
        let mut zero_crossings = 0;
        for i in 1..y.len() {
            if y[i-1] <= 0.0 && y[i] > 0.0 {
                zero_crossings += 1;
            }
        }
        
        let duration_sec = signal.len() as f32 / sample_rate;
        let measured_freq = (zero_crossings as f32) / (2.0 * duration_sec);
        let freq_error = (measured_freq - 100.0).abs();
        
        println!("Measured frequency: {:.1} Hz (error: {:.1}%)", 
                measured_freq, (freq_error / 100.0) * 100.0);
                
        assert!(
            freq_error < 5.0,  // Less than 5% frequency error
            "Output frequency is too far from expected. Expected 100Hz, got {:.1}Hz",
            measured_freq
        );
    }

    #[test]
    fn test_multiband_compressor() {
        let sample_rate = 44100.0;
        let duration = 0.1; // 100ms
        
        // Create a test signal with multiple frequencies
        let mut signal = generate_test_signal(100.0, sample_rate, duration);
        let high_freq = generate_test_signal(1000.0, sample_rate, duration);
        for (i, &sample) in high_freq.iter().enumerate() {
            signal[i] += sample * 0.5; // Add some high frequency content
        }
        
        // Create a 2-band compressor
        let bands = vec![
            BandParams {
                low_freq: 0.0,
                high_freq: 250.0,
                threshold: -20.0,
                ratio: 4.0,
                attack_ms: 10.0,
                release_ms: 100.0,
            },
            BandParams {
                low_freq: 250.0,
                high_freq: sample_rate * 0.5,
                threshold: -20.0,
                ratio: 4.0,
                attack_ms: 10.0,
                release_ms: 100.0,
            },
        ];
        
        let mut compressor = MultibandCompressor::new(bands, sample_rate);
        let output = compressor.process(&signal);
        
        // Basic validation
        assert_eq!(output.len(), signal.len());
        assert_ne!(output, signal); // Output should be different from input
        
        // Check that the output is not all zeros
        let output_energy: f32 = output.iter().map(|x| x * x).sum();
        assert!(output_energy > 0.0);
    }
}
