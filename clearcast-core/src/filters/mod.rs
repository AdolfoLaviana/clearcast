//! Audio filters for ClearCast

mod compressor;

pub use compressor::compress_rms;

/// Applies a simple gain to the audio signal
/// 
/// # Arguments
/// * `input` - Input audio buffer
/// * `gain` - Gain factor to apply (1.0 = no change)
/// 
/// # Returns
/// New buffer with gain applied
/// 
/// # Example
/// ```
/// use clearcast_core::filters::apply_gain;
/// let input = vec![1.0, 0.5, -0.5, -1.0];
/// let output = apply_gain(&input, 2.0);
/// assert_eq!(output, vec![2.0, 1.0, -1.0, -2.0]);
/// ```
pub fn apply_gain(input: &[f32], gain: f32) -> Vec<f32> {
    input.iter().map(|x| x * gain).collect()
}

/// Applies a simple low-pass filter (first-order IIR)
/// 
/// # Arguments
/// * `input` - Input audio buffer
/// * `alpha` - Smoothing factor (0.0 to 1.0, higher = more smoothing)
/// 
/// # Returns
/// New buffer with low-pass filter applied
pub fn low_pass(input: &[f32], alpha: f32) -> Vec<f32> {
    if input.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(input.len());
    let mut prev = input[0];
    
    for &sample in input {
        let filtered = prev + alpha * (sample - prev);
        result.push(filtered);
        prev = filtered;
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_apply_gain() {
        let input = vec![1.0, 0.5, -0.5, -1.0];
        let expected = vec![2.0, 1.0, -1.0, -2.0];
        let result = apply_gain(&input, 2.0);
        assert_eq!(result, expected);
    }

    #[wasm_bindgen_test]
    fn test_low_pass() {
        let input = vec![0.0, 1.0, 0.0, 1.0, 0.0];
        let result = low_pass(&input, 0.5);
        assert_eq!(result.len(), input.len());
        // The first value should be the same
        assert_eq!(result[0], 0.0);
    }
}
