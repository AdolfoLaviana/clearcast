use clearcast_core::filters::wiener_filter::{reduce_noise_wiener, estimate_noise_profile};
use rand::Rng;
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
    let mut rng = rand::thread_rng();
    (0..num_samples)
        .map(|_| (rng.gen::<f32>() - 0.5) * 2.0 * amplitude)
        .collect()
}

// Helper function to generate an impulse
fn generate_impulse(num_samples: usize, position: usize, amplitude: f32) -> Vec<f32> {
    let mut signal = vec![0.0; num_samples];
    if position < num_samples {
        signal[position] = amplitude;
    }
    signal
}

// Helper function to add noise to a signal
fn add_noise(signal: &[f32], noise_amplitude: f32) -> Vec<f32> {
    let noise = generate_white_noise(signal.len(), noise_amplitude);
    signal.iter().zip(noise.iter()).map(|(&s, &n)| s + n).collect()
}

// Helper function to calculate RMS (Root Mean Square)
fn calculate_rms(signal: &[f32]) -> f32 {
    let sum_sq: f32 = signal.iter().map(|&x| x * x).sum();
    (sum_sq / signal.len() as f32).sqrt()
}

#[test]
fn test_reduce_noise_sine_wave() {
    // Generate a clean sine wave
    let sample_rate = 44100.0;
    let duration = 0.1; // 100ms
    let freq = 1000.0;  // 1kHz
    
    let clean_signal = generate_sine_wave(freq, sample_rate, duration);
    let noise_amplitude = 0.2;
    let noisy_signal = add_noise(&clean_signal, noise_amplitude);
    
    // Estimate noise profile from a silent section
    let noise_profile = vec![noise_amplitude; 2048 / 2 + 1];
    
    // Apply Wiener filter
    let fft_size = 2048;
    let hop_size = fft_size / 4;
    let processed_signal = reduce_noise_wiener(
        &noisy_signal,
        &noise_profile,
        fft_size,
        hop_size,
        0.9,
    );
    
    // Calculate RMS values
    let noise_rms = calculate_rms(
        &noisy_signal
            .iter()
            .zip(clean_signal.iter())
            .map(|(&n, &c)| n - c)
            .collect::<Vec<_>>(),
    );
    
    let residual_noise_rms = calculate_rms(
        &processed_signal
            .iter()
            .zip(clean_signal.iter())
            .map(|(&p, &c)| p - c)
            .collect::<Vec<_>>(),
    );
    
    // Check that noise was reduced
    assert!(
        residual_noise_rms < noise_rms * 0.7, // At least 30% reduction
        "Noise not sufficiently reduced: {} >= {}",
        residual_noise_rms,
        noise_rms * 0.7
    );
    
    // Check that the signal was preserved
    let clean_rms = calculate_rms(&clean_signal);
    let processed_rms = calculate_rms(&processed_signal);
    
    assert!(
        (clean_rms - processed_rms).abs() < clean_rms * 0.1, // Within 10% of original
        "Signal changed too much: {} vs {}",
        clean_rms,
        processed_rms
    );
}

#[test]
fn test_impulse_response() {
    // Test how the filter handles impulse responses
    let num_samples = 1024;
    let impulse_pos = num_samples / 2;
    let impulse_amplitude = 1.0;
    
    let impulse = generate_impulse(num_samples, impulse_pos, impulse_amplitude);
    let noise_amplitude = 0.1;
    let noisy_impulse = add_noise(&impulse, noise_amplitude);
    
    let noise_profile = vec![noise_amplitude; 512 / 2 + 1];
    let fft_size = 512;
    let hop_size = fft_size / 4;
    
    let processed = reduce_noise_wiener(
        &noisy_impulse,
        &noise_profile,
        fft_size,
        hop_size,
        0.85,
    );
    
    // The impulse should still be clearly visible in the output
    let max_processed = processed
        .iter()
        .fold(0.0, |max, &x| if x.abs() > max { x.abs() } else { max });
    
    assert!(
        max_processed > impulse_amplitude * 0.5, // At least 50% of original
        "Impulse not preserved: {}",
        max_processed
    );
    
    // The noise floor should be lower
    let noise_floor_before = calculate_rms(&noisy_impulse[..impulse_pos]);
    let noise_floor_after = calculate_rms(&processed[..impulse_pos]);
    
    assert!(
        noise_floor_after < noise_floor_before * 0.8, // At least 20% reduction
        "Noise floor not reduced: {} >= {}",
        noise_floor_after,
        noise_floor_before * 0.8
    );
}

#[test]
fn test_noise_estimation() {
    // Test the noise profile estimation
    let sample_rate = 44100.0;
    let duration = 0.1; // 100ms
    let num_samples = (sample_rate * duration) as usize;
    
    // Generate noise with known characteristics
    let noise_amplitude = 0.15;
    let noise = generate_white_noise(num_samples, noise_amplitude);
    
    // Estimate the noise profile
    let fft_size = 1024;
    let estimated_profile = estimate_noise_profile(&noise, fft_size);
    
    // The estimated profile should be relatively flat
    let mean_amplitude = estimated_profile.iter().sum::<f32>() / estimated_profile.len() as f32;
    
    // Check that the mean is close to the expected noise amplitude
    assert!(
        (mean_amplitude - noise_amplitude).abs() < noise_amplitude * 0.3, // Within 30%
        "Estimated noise amplitude {} too far from expected {}",
        mean_amplitude,
        noise_amplitude
    );
    
    // Check that the profile is relatively flat (standard deviation is small)
    let variance = estimated_profile
        .iter()
        .map(|&x| (x - mean_amplitude).powi(2))
        .sum::<f32>()
        / estimated_profile.len() as f32;
    let std_dev = variance.sqrt();
    
    assert!(
        std_dev < mean_amplitude * 0.5, // Standard deviation less than 50% of mean
        "Noise profile not flat enough: std_dev = {}, mean = {}",
        std_dev,
        mean_amplitude
    );
}

#[test]
fn test_edge_cases() {
    // Test with empty input
    let empty: Vec<f32> = vec![];
    let noise_profile = vec![0.1; 1024 / 2 + 1];
    
    let result = reduce_noise_wiener(&empty, &noise_profile, 1024, 256, 0.9);
    assert!(result.is_empty(), "Should return empty vec for empty input");
    
    // Test with very short input
    let short_input = vec![0.1, -0.1, 0.05];
    let result = reduce_noise_wiener(&short_input, &noise_profile, 1024, 256, 0.9);
    assert_eq!(result.len(), short_input.len(), "Output length should match input length");
    
    // Test with zero noise profile
    let zero_noise = vec![0.0; 1024 / 2 + 1];
    let result = reduce_noise_wiener(&short_input, &zero_noise, 1024, 256, 0.9);
    // Should still process without panicking
    assert_eq!(result.len(), short_input.len());
}
