//! Benchmark suite for ClearCast audio processing
//! 
//! This benchmark measures the performance of audio processing with different buffer sizes
//! to help identify optimal buffer sizes for different use cases.

use clearcast_core::AudioEngine;
use criterion::{
    criterion_group, criterion_main, BatchSize, Criterion, Throughput,
};
use rand::Rng;
use std::time::Duration;

/// Generate a vector of random audio samples with values between -1.0 and 1.0
fn generate_audio_samples(size: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

/// Benchmark the process function with different buffer sizes
fn benchmark_processing(c: &mut Criterion) {
    let processor = AudioEngine::new();
    
    // Test with buffer sizes from 2^8 (256) to 2^16 (65536) samples
    // which covers common audio buffer sizes
    for size in (8..=16).map(|n| 1 << n) {
        let input = generate_audio_samples(size);
        
        let mut group = c.benchmark_group(format!("process/{}_samples", size));
        group.throughput(Throughput::Elements(size as u64));
        
        // Measure processing time
        group.bench_function("process", |b| {
            b.iter_batched(
                || input.clone(),
                |data| processor.process(data).unwrap(),
                BatchSize::SmallInput,
            )
        });
        
        // Measure memory usage and throughput
        group.bench_function("process_with_metrics", |b| {
            b.iter(|| {
                let output = processor.process(input.clone()).unwrap();
                criterion::black_box(output);
            })
        });
        
        group.finish();
    }
}

/// Benchmark the normalize function with different input sizes
fn benchmark_normalize(c: &mut Criterion) {
    for size in (8..=16).step_by(2).map(|n| 1 << n) {
        let input = generate_audio_samples(size);
        
        let mut group = c.benchmark_group(format!("normalize/{}_samples", size));
        group.throughput(Throughput::Elements(size as u64));
        
        // Test with default settings
        group.bench_function("process_with_defaults", |b| {
            let processor = AudioEngine::new();
            b.iter(|| {
                let output = processor.process(input.clone()).unwrap();
                criterion::black_box(output);
            })
        });
        
        // Test with custom settings
        group.bench_function("process_with_custom_settings", |b| {
            let processor = AudioEngine::with_settings(0.05, 0.8).unwrap();
            b.iter(|| {
                let output = processor.process(input.clone()).unwrap();
                criterion::black_box(output);
            })
        });
        
        group.finish();
    }
}

/// Benchmark the noise reduction with different input sizes and settings
fn benchmark_noise_reduction(c: &mut Criterion) {
    for size in (8..=16).step_by(2).map(|n| 1 << n) {
        let input = generate_audio_samples(size);
        
        let mut group = c.benchmark_group(format!("noise_reduction/{}_samples", size));
        group.throughput(Throughput::Elements(size as u64));
        
        // Test with different noise reduction thresholds
        for &threshold in &[0.01, 0.05, 0.1] {
            group.bench_function(
                format!("noise_reduction_threshold_{}", threshold).as_str(),
                |b| {
                    let processor = AudioEngine::with_settings(threshold, 1.0).unwrap();
                    b.iter(|| {
                        let output = processor.process(input.clone()).unwrap();
                        criterion::black_box(output);
                    })
                },
            );
        }
        
        group.finish();
    }
}

// Configuration for benchmark groups
criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)  // Increase sample size for more stable results
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(5))
        .noise_threshold(0.05);  // 5% noise threshold for statistical significance
    targets = 
        benchmark_processing,
        benchmark_normalize,
        benchmark_noise_reduction
}

criterion_main!(benches);
