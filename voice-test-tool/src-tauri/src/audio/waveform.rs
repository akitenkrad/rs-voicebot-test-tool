//! Waveform data generator
//!
//! Generates downsampled waveform peak data for UI visualization.

/// Generate waveform peaks for visualization.
///
/// Downsamples the audio to `resolution` points and returns a Vec<f32>
/// of peak amplitude values normalized to 0.0..1.0.
///
/// If the audio is multi-channel, it will be mixed down to mono first.
///
/// # Arguments
/// * `samples` - Interleaved audio samples (f32, -1.0..1.0)
/// * `channels` - Number of audio channels
/// * `resolution` - Number of waveform data points to generate
///
/// # Returns
/// A Vec<f32> of `resolution` peak values, each in the range 0.0..1.0
pub fn generate_waveform_peaks(samples: &[f32], channels: u16, resolution: usize) -> Vec<f32> {
    if samples.is_empty() || resolution == 0 {
        return vec![0.0; resolution];
    }

    let channels = channels.max(1) as usize;

    // Mix down to mono if multi-channel
    let mono_samples: Vec<f32> = if channels > 1 {
        samples
            .chunks_exact(channels)
            .map(|frame| {
                let sum: f32 = frame.iter().sum();
                sum / channels as f32
            })
            .collect()
    } else {
        samples.to_vec()
    };

    let total_mono = mono_samples.len();
    if total_mono == 0 {
        return vec![0.0; resolution];
    }

    let samples_per_segment = total_mono as f64 / resolution as f64;

    let mut peaks = Vec::with_capacity(resolution);

    for i in 0..resolution {
        let start = (i as f64 * samples_per_segment) as usize;
        let end = (((i + 1) as f64) * samples_per_segment) as usize;
        let end = end.min(total_mono);

        if start >= end {
            peaks.push(0.0);
            continue;
        }

        // Find the peak (max absolute value) in this segment
        let peak = mono_samples[start..end]
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);

        // Clamp to 0.0..1.0
        peaks.push(peak.min(1.0));
    }

    peaks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_samples() {
        let result = generate_waveform_peaks(&[], 1, 100);
        assert_eq!(result.len(), 100);
        assert!(result.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_zero_resolution() {
        let samples = vec![0.5, -0.5, 0.3];
        let result = generate_waveform_peaks(&samples, 1, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_mono_peaks() {
        // 10 samples, resolution 2 -> 5 samples per segment
        let samples = vec![0.1, 0.5, 0.3, 0.2, 0.4, -0.8, 0.1, 0.2, 0.3, 0.1];
        let result = generate_waveform_peaks(&samples, 1, 2);
        assert_eq!(result.len(), 2);
        // First segment: [0.1, 0.5, 0.3, 0.2, 0.4] -> peak = 0.5
        assert!((result[0] - 0.5).abs() < 0.001);
        // Second segment: [-0.8, 0.1, 0.2, 0.3, 0.1] -> peak = 0.8
        assert!((result[1] - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_stereo_mixdown() {
        // 4 stereo frames (8 samples), resolution 2
        // Frame 0: (0.6, 0.4) -> mono 0.5
        // Frame 1: (0.2, 0.8) -> mono 0.5
        // Frame 2: (-1.0, 0.0) -> mono -0.5
        // Frame 3: (0.0, 0.0) -> mono 0.0
        let samples = vec![0.6, 0.4, 0.2, 0.8, -1.0, 0.0, 0.0, 0.0];
        let result = generate_waveform_peaks(&samples, 2, 2);
        assert_eq!(result.len(), 2);
        // First segment: [0.5, 0.5] -> peak = 0.5
        assert!((result[0] - 0.5).abs() < 0.001);
        // Second segment: [-0.5, 0.0] -> peak = 0.5
        assert!((result[1] - 0.5).abs() < 0.001);
    }
}
