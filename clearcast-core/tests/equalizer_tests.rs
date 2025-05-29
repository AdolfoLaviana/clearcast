use clearcast_core::filters::equalizer::{parametric_eq, Band};
use std::f32::consts::PI;

// Helper function to generate a sine wave
fn generate_sine_wave(freq: f32, sample_rate: f32, duration_secs: f32) -> Vec<f32> {
    let num_samples = (sample_rate * duration_secs) as usize;
    (0..num_samples)
        .map(|i| (2.0 * PI * freq * i as f32 / sample_rate).sin())
        .collect()
}

// Helper function to generate white noise
fn generate_white_noise(num_samples: usize, amplitude: f32) -> Vec<f32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..num_samples)
        .map(|_| (rng.gen::<f32>() - 0.5) * 2.0 * amplitude)
        .collect()
}

// Helper function to calculate RMS (Root Mean Square)
fn calculate_rms(signal: &[f32]) -> f32 {
    let sum_sq: f32 = signal.iter().map(|&x| x * x).sum();
    (sum_sq / signal.len() as f32).sqrt()
}

// Helper function to calculate frequency spectrum using FFT
fn calculate_spectrum(signal: &[f32], sample_rate: f32) -> Vec<(f32, f32)> {
    use rustfft::{FftPlanner, num_complex::Complex};
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(signal.len());
    
    // Convert to complex numbers
    let mut buffer: Vec<Complex<f32>> = signal.iter().map(|&x| Complex { re: x, im: 0.0 }).collect();
    
    // Perform FFT
    fft.process(&mut buffer);
    
    // Calculate magnitude spectrum
    let n = buffer.len();
    let fft_bin_width = sample_rate / n as f32;
    
    (0..n/2).map(|i| {
        let freq = i as f32 * fft_bin_width;
        let mag = (buffer[i].re.powi(2) + buffer[i].im.powi(2)).sqrt() / (n as f32).sqrt();
        (freq, mag)
    }).collect()
}

// Helper function to find peak frequency in a spectrum
fn find_peak_frequency(spectrum: &[(f32, f32)]) -> f32 {
    spectrum.iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|&(freq, _)| freq)
        .unwrap_or(0.0)
}

#[test]
fn test_parametric_eq_sine_wave() {
    let sample_rate = 44100.0;
    let duration = 0.1; // 100ms
    
    // Test low frequency boost
    let freq_low = 100.0;  // Low frequency
    let signal_low = generate_sine_wave(freq_low, sample_rate, duration);
    
    // Apply 6dB boost to low frequencies
    let processed = parametric_eq(&signal_low, sample_rate, 6.0, 0.0, 0.0);
    
    // The RMS should increase by approximately 6dB (2x amplitude)
    let original_rms = calculate_rms(&signal_low);
    let processed_rms = calculate_rms(&processed);
    let gain_db = 20.0 * (processed_rms / original_rms).log10();
    
    assert!(
        (gain_db - 6.0).abs() < 1.0, // Within 1dB of expected gain
        "Expected ~6dB gain, got {}dB",
        gain_db
    );
    
    // Test high frequency boost
    let freq_high = 5000.0;  // High frequency
    let signal_high = generate_sine_wave(freq_high, sample_rate, duration);
    
    // Apply 6dB boost to high frequencies
    let processed = parametric_eq(&signal_high, sample_rate, 0.0, 0.0, 6.0);
    
    // The RMS should increase by approximately 6dB (2x amplitude)
    let original_rms = calculate_rms(&signal_high);
    let processed_rms = calculate_rms(&processed);
    let gain_db = 20.0 * (processed_rms / original_rms).log10();
    
    assert!(
        (gain_db - 6.0).abs() < 1.0, // Within 1dB of expected gain
        "Expected ~6dB gain, got {}dB",
        gain_db
    );
}

#[test]
fn test_parametric_eq_frequency_response() {
    let sample_rate = 44100.0;
    let duration = 0.1; // 100ms
    
    // Generate a test signal with multiple frequencies
    let freq_low = 100.0;
    let freq_mid = 1000.0;
    let freq_high = 5000.0;
    
    let signal_low = generate_sine_wave(freq_low, sample_rate, duration);
    let signal_mid = generate_sine_wave(freq_mid, sample_rate, duration);
    let signal_high = generate_sine_wave(freq_high, sample_rate, duration);
    
    // Combine the signals
    let combined: Vec<f32> = signal_low.iter()
        .zip(signal_mid.iter())
        .zip(signal_high.iter())
        .map(|((&l, &m), &h)| l + m + h)
        .collect();
    
    // Apply EQ with different gains for each band
    let boost_db = 12.0; // 12dB boost
    let processed = parametric_eq(&combined, sample_rate, boost_db, -boost_db/2.0, boost_db);
    
    // Calculate frequency spectrum of the processed signal
    let spectrum = calculate_spectrum(&processed, sample_rate);
    
    // Find peaks in different frequency ranges
    let low_band: Vec<_> = spectrum.iter().filter(|(f, _)| *f < 200.0).collect();
    let mid_band: Vec<_> = spectrum.iter().filter(|(f, _)| *f >= 200.0 && *f <= 3000.0).collect();
    let high_band: Vec<_> = spectrum.iter().filter(|(f, _)| *f > 3000.0).collect();
    
    // Calculate average magnitude in each band
    let avg_low = low_band.iter().map(|(_, m)| m).sum::<f32>() / low_band.len() as f32;
    let avg_mid = mid_band.iter().map(|(_, m)| m).sum::<f32>() / mid_band.len() as f32;
    let avg_high = high_band.iter().map(|(_, m)| m).sum::<f32>() / high_band.len() as f32;
    
    // Calculate relative gains between bands
    let low_to_mid = 20.0 * (avg_low / avg_mid).log10();
    let high_to_mid = 20.0 * (avg_high / avg_mid).log10();
    
    // Check that the relative gains match our EQ settings
    // We expect low band to be 18dB higher than mid (12 - (-6))
    // And high band to be 18dB higher than mid (12 - (-6))
    assert!(
        (low_to_mid - 18.0).abs() < 3.0, // Within 3dB of expected
        "Low to mid gain difference {}dB not as expected",
        low_to_mid
    );
    
    assert!(
        (high_to_mid - 18.0).abs() < 3.0, // Within 3dB of expected
        "High to mid gain difference {}dB not as expected",
        high_to_mid
    );
}

#[test]
fn test_parametric_eq_noise() {
    // Test with white noise to verify frequency response
    let sample_rate = 44100.0;
    let num_samples = 2usize.pow(15); // 32768 samples (~0.74s at 44.1kHz)
    
    // Generate white noise
    let noise = generate_white_noise(num_samples, 0.5);
    
    // Apply EQ with known settings
    let low_gain = 6.0;    // +6dB
    let mid_gain = -12.0;  // -12dB
    let high_gain = 0.0;   // 0dB
    
    let processed = parametric_eq(&noise, sample_rate, low_gain, mid_gain, high_gain);
    
    // Calculate frequency spectrum
    let spectrum = calculate_spectrum(&processed, sample_rate);
    
    // Calculate average magnitude in each band
    let (low_sum, low_count) = spectrum.iter()
        .filter(|(f, _)| *f < 200.0)
        .fold((0.0, 0), |(sum, count), (_, m)| (sum + m, count + 1));
    
    let (mid_sum, mid_count) = spectrum.iter()
        .filter(|(f, _)| *f >= 200.0 && *f <= 3000.0)
        .fold((0.0, 0), |(sum, count), (_, m)| (sum + m, count + 1));
    
    let (high_sum, high_count) = spectrum.iter()
        .filter(|(f, _)| *f > 3000.0)
        .fold((0.0, 0), |(sum, count), (_, m)| (sum + m, count + 1));
    
    let avg_low = low_sum / low_count as f32;
    let avg_mid = mid_sum / mid_count as f32;
    let avg_high = high_sum / high_count as f32;
    
    // Calculate relative gains between bands (in dB)
    let low_to_mid = 20.0 * (avg_low / avg_mid).log10();
    let high_to_mid = 20.0 * (avg_high / avg_mid).log10();
    
    // Check that the relative gains are close to what we set
    // Expected: low is 18dB higher than mid (6 - (-12))
    //           high is 12dB higher than mid (0 - (-12))
    assert!(
        (low_to_mid - 18.0).abs() < 3.0, // Within 3dB
        "Low to mid gain difference {}dB not as expected",
        low_to_mid
    );
    
    assert!(
        (high_to_mid - 12.0).abs() < 3.0, // Within 3dB
        "High to mid gain difference {}dB not as expected",
        high_to_mid
    );
}

#[test]
fn test_edge_cases() {
    let sample_rate = 44100.0;
    
    // Test with empty input
    let empty: Vec<f32> = vec![];
    let result = parametric_eq(&empty, sample_rate, 0.0, 0.0, 0.0);
    assert!(result.is_empty(), "Should return empty vec for empty input");
    
    // Test with very short input
    let short_input = vec![0.1, -0.1, 0.05];
    let result = parametric_eq(&short_input, sample_rate, 0.0, 0.0, 0.0);
    assert_eq!(result.len(), short_input.len(), "Output length should match input length");
    
    // Test with extreme gain values
    let extreme_boost = 48.0; // 48dB boost (very high)
    let extreme_cut = -48.0;  // 48dB cut (very low)
    
    let signal = generate_sine_wave(1000.0, sample_rate, 0.01);
    
    // Extreme boost
    let boosted = parametric_eq(&signal, sample_rate, extreme_boost, 0.0, 0.0);
    let boosted_rms = calculate_rms(&boosted);
    let original_rms = calculate_rms(&signal);
    let actual_boost = 20.0 * (boosted_rms / original_rms).log10();
    
    // Should be close to the requested boost, but may be limited by internal processing
    assert!(
        actual_boost > extreme_boost * 0.8, // At least 80% of requested boost
        "Extreme boost not applied: {}dB < {}dB",
        actual_boost,
        extreme_boost * 0.8
    );
    
    // Extreme cut
    let cut = parametric_eq(&signal, sample_rate, extreme_cut, 0.0, 0.0);
    let cut_rms = calculate_rms(&cut);
    
    // Signal should be significantly attenuated
    assert!(
        cut_rms < original_rms * 0.01, // At least 99% reduction
        "Extreme cut not applied: {} >= {}",
        cut_rms,
        original_rms * 0.01
    );
}
