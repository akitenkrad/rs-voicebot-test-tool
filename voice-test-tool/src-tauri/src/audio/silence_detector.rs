//! Silence detection for audio streams.

/// Silence detector that tracks consecutive silent samples.
pub struct SilenceDetector {
    /// Amplitude threshold below which samples are considered silent
    threshold_amplitude: f32,
    /// Number of consecutive silent samples required to trigger detection
    required_samples: usize,
    /// Current count of consecutive silent samples
    silence_samples: usize,
}

impl SilenceDetector {
    /// Create a new silence detector.
    ///
    /// # Arguments
    /// * `threshold_db` - Threshold in decibels (e.g., -40.0 dB)
    /// * `duration_ms` - Required silence duration in milliseconds
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(threshold_db: f32, duration_ms: u64, sample_rate: u32) -> Self {
        // Convert dB threshold to linear amplitude (0.0 to 1.0)
        // dB = 20 * log10(amplitude)
        // amplitude = 10^(dB/20)
        let threshold_amplitude = 10f32.powf(threshold_db / 20.0);
        let required_samples = ((duration_ms as f32 / 1000.0) * sample_rate as f32) as usize;

        Self {
            threshold_amplitude,
            required_samples,
            silence_samples: 0,
        }
    }

    /// Reset the silence counter.
    pub fn reset(&mut self) {
        self.silence_samples = 0;
    }

    /// Process a batch of samples and return true if silence duration threshold is reached.
    ///
    /// # Arguments
    /// * `samples` - Audio samples in the range [-1.0, 1.0]
    ///
    /// # Returns
    /// `true` if the required silence duration has been reached, `false` otherwise.
    pub fn process(&mut self, samples: &[f32]) -> bool {
        for &sample in samples {
            if sample.abs() < self.threshold_amplitude {
                self.silence_samples += 1;
                if self.silence_samples >= self.required_samples {
                    return true;
                }
            } else {
                self.silence_samples = 0;
            }
        }
        false
    }

    /// Get the current silence duration in samples.
    pub fn current_silence_samples(&self) -> usize {
        self.silence_samples
    }

    /// Get the required silence duration in samples.
    pub fn required_silence_samples(&self) -> usize {
        self.required_samples
    }

    /// Get the silence progress as a percentage (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
        if self.required_samples == 0 {
            return 1.0;
        }
        (self.silence_samples as f32 / self.required_samples as f32).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_detection() {
        // Create detector: -40dB threshold, 100ms duration, 1000 Hz sample rate
        // This means we need 100 consecutive silent samples
        let mut detector = SilenceDetector::new(-40.0, 100, 1000);

        // Silent samples (below -40dB threshold)
        let silent_samples: Vec<f32> = vec![0.001; 50];
        assert!(!detector.process(&silent_samples)); // Not enough yet

        // More silent samples
        assert!(detector.process(&silent_samples)); // Should trigger at 100 samples
    }

    #[test]
    fn test_silence_reset_on_sound() {
        let mut detector = SilenceDetector::new(-40.0, 100, 1000);

        // Silent samples
        let silent_samples: Vec<f32> = vec![0.001; 50];
        detector.process(&silent_samples);
        assert_eq!(detector.current_silence_samples(), 50);

        // Loud sample resets counter
        let loud_samples: Vec<f32> = vec![0.5];
        detector.process(&loud_samples);
        assert_eq!(detector.current_silence_samples(), 0);
    }

    #[test]
    fn test_threshold_conversion() {
        // -40dB should be approximately 0.01 amplitude
        let detector = SilenceDetector::new(-40.0, 100, 1000);
        // threshold_amplitude = 10^(-40/20) = 10^(-2) = 0.01
        assert!((detector.threshold_amplitude - 0.01).abs() < 0.0001);
    }
}
