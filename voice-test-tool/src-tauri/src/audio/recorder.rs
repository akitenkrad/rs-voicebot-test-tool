//! Audio recorder using cpal for input capture.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use serde::Serialize;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::recording::RecordingConfig;
use super::silence_detector::SilenceDetector;

/// Information about an audio device.
#[derive(Debug, Clone, Serialize)]
pub struct InputDeviceInfo {
    /// Device name
    pub name: String,
    /// Whether this is the default device
    pub is_default: bool,
    /// Device type: "input" for microphones, "output" for speakers/loopback
    pub device_type: String,
}

/// Audio recorder for capturing audio from input devices.
pub struct AudioRecorder {
    config: RecordingConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
    is_recording: Arc<AtomicBool>,
    silence_detected: Arc<AtomicBool>,
}

impl AudioRecorder {
    /// Create a new audio recorder with the given configuration.
    pub fn new(config: RecordingConfig) -> Self {
        Self {
            config,
            buffer: Arc::new(Mutex::new(Vec::new())),
            is_recording: Arc::new(AtomicBool::new(false)),
            silence_detected: Arc::new(AtomicBool::new(false)),
        }
    }

    /// List available audio devices (both input and output).
    /// Output devices are included for loopback recording (capturing system audio).
    pub fn list_input_devices() -> Result<Vec<InputDeviceInfo>, String> {
        let host = cpal::default_host();
        let mut result = Vec::new();

        // Get default device names for comparison
        let default_input_name = host
            .default_input_device()
            .and_then(|d| d.name().ok());
        let default_output_name = host
            .default_output_device()
            .and_then(|d| d.name().ok());

        // Add input devices (microphones)
        if let Ok(devices) = host.input_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    let is_default = default_input_name
                        .as_ref()
                        .map(|default| default == &name)
                        .unwrap_or(false);
                    result.push(InputDeviceInfo {
                        name,
                        is_default,
                        device_type: "input".to_string(),
                    });
                }
            }
        }

        // Add output devices (for loopback recording)
        if let Ok(devices) = host.output_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    // Skip if already added as input device
                    if result.iter().any(|d| d.name == name) {
                        continue;
                    }
                    let is_default = default_output_name
                        .as_ref()
                        .map(|default| default == &name)
                        .unwrap_or(false);
                    result.push(InputDeviceInfo {
                        name,
                        is_default,
                        device_type: "output".to_string(),
                    });
                }
            }
        }

        Ok(result)
    }

    /// Find an input device by name, or return the default device if name is None.
    fn get_device(&self) -> Result<cpal::Device, String> {
        let host = cpal::default_host();

        match &self.config.device_name {
            Some(name) => {
                let devices = host
                    .input_devices()
                    .map_err(|e| format!("Failed to enumerate input devices: {}", e))?;

                for device in devices {
                    if let Ok(device_name) = device.name() {
                        if device_name == *name {
                            return Ok(device);
                        }
                    }
                }
                Err(format!("Input device '{}' not found", name))
            }
            None => host
                .default_input_device()
                .ok_or_else(|| "No default input device available".to_string()),
        }
    }

    /// Start recording audio.
    ///
    /// Returns the cpal Stream that must be kept alive while recording.
    /// The stream will automatically stop when dropped.
    pub fn start_recording(&mut self) -> Result<cpal::Stream, String> {
        // Reset state
        self.buffer.lock().clear();
        self.is_recording.store(true, Ordering::SeqCst);
        self.silence_detected.store(false, Ordering::SeqCst);

        let device = self.get_device()?;
        tracing::info!("Recording from device: {:?}", device.name());

        // Get supported config
        let supported_configs = device
            .supported_input_configs()
            .map_err(|e| format!("Failed to get supported configs: {}", e))?;

        // Try to find a config matching our requirements
        let config = self.find_suitable_config(supported_configs)?;
        tracing::info!(
            "Recording config: {} Hz, {} channels",
            config.sample_rate().0,
            config.channels()
        );

        // Create silence detector
        let silence_detector = Arc::new(Mutex::new(SilenceDetector::new(
            self.config.silence_threshold_db,
            self.config.silence_duration_ms,
            config.sample_rate().0,
        )));

        // Clone Arcs for the callback
        let buffer = Arc::clone(&self.buffer);
        let is_recording = Arc::clone(&self.is_recording);
        let silence_detected = Arc::clone(&self.silence_detected);
        let max_samples =
            (self.config.max_recording_sec as f32 * config.sample_rate().0 as f32) as usize;

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if !is_recording.load(Ordering::SeqCst) {
                        return;
                    }

                    let mut buf = buffer.lock();

                    // Check max recording duration
                    if buf.len() >= max_samples {
                        is_recording.store(false, Ordering::SeqCst);
                        tracing::info!("Max recording duration reached");
                        return;
                    }

                    // Add samples to buffer
                    buf.extend_from_slice(data);

                    // Check for silence
                    let mut detector = silence_detector.lock();
                    if detector.process(data) {
                        silence_detected.store(true, Ordering::SeqCst);
                        tracing::info!("Silence detected, stopping recording");
                    }
                },
                |err| {
                    tracing::error!("Recording error: {}", err);
                },
                None,
            )
            .map_err(|e| format!("Failed to build input stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start recording: {}", e))?;

        Ok(stream)
    }

    /// Find a suitable stream config from the supported configs.
    fn find_suitable_config(
        &self,
        configs: cpal::SupportedInputConfigs,
    ) -> Result<cpal::SupportedStreamConfig, String> {
        let target_sample_rate = cpal::SampleRate(self.config.sample_rate);
        let target_channels = self.config.channels;

        let mut best_config: Option<cpal::SupportedStreamConfig> = None;

        for config in configs {
            // Check channel count
            if config.channels() != target_channels {
                continue;
            }

            // Check if sample rate is supported
            if config.min_sample_rate() <= target_sample_rate
                && config.max_sample_rate() >= target_sample_rate
            {
                return Ok(config.with_sample_rate(target_sample_rate));
            }

            // Store as fallback if channels match
            if best_config.is_none() {
                best_config = Some(config.with_max_sample_rate());
            }
        }

        // If we found a config with matching channels but different sample rate
        if let Some(config) = best_config {
            return Ok(config);
        }

        // Last resort: get any supported config
        let device = self.get_device()?;
        device
            .default_input_config()
            .map_err(|e| format!("No suitable input config found: {}", e))
    }

    /// Stop recording and return the recorded samples.
    pub fn stop_recording(&mut self) -> Vec<f32> {
        self.is_recording.store(false, Ordering::SeqCst);
        let mut buffer = self.buffer.lock();
        std::mem::take(&mut *buffer)
    }

    /// Check if currently recording.
    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    /// Check if silence has been detected.
    pub fn is_silence_detected(&self) -> bool {
        self.silence_detected.load(Ordering::SeqCst)
    }

    /// Get a clone of the silence_detected flag for cross-thread monitoring.
    pub fn silence_detected_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.silence_detected)
    }

    /// Get a clone of the is_recording flag for cross-thread monitoring.
    pub fn is_recording_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_recording)
    }

    /// Get the current recording buffer length in samples.
    pub fn buffer_length(&self) -> usize {
        self.buffer.lock().len()
    }

    /// Save recorded samples as a WAV file.
    ///
    /// # Arguments
    /// * `samples` - Audio samples in f32 format
    /// * `sample_rate` - Sample rate in Hz
    /// * `channels` - Number of audio channels
    /// * `path` - Output file path
    pub fn save_as_wav(
        samples: &[f32],
        sample_rate: u32,
        channels: u16,
        path: &Path,
    ) -> Result<(), String> {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer =
            hound::WavWriter::create(path, spec).map_err(|e| format!("Failed to create WAV file: {}", e))?;

        // Convert f32 samples to i16
        for &sample in samples {
            let sample_i16 = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            writer
                .write_sample(sample_i16)
                .map_err(|e| format!("Failed to write sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV file: {}", e))?;

        Ok(())
    }

    /// Convert recorded samples to WAV bytes in memory.
    ///
    /// # Arguments
    /// * `samples` - Audio samples in f32 format
    /// * `sample_rate` - Sample rate in Hz
    /// * `channels` - Number of audio channels
    pub fn samples_to_wav_bytes(
        samples: &[f32],
        sample_rate: u32,
        channels: u16,
    ) -> Result<Vec<u8>, String> {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut cursor = std::io::Cursor::new(Vec::new());
        let mut writer = hound::WavWriter::new(&mut cursor, spec)
            .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

        for &sample in samples {
            let sample_i16 =
                (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            writer
                .write_sample(sample_i16)
                .map_err(|e| format!("Failed to write sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV: {}", e))?;

        Ok(cursor.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_as_wav() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.wav");

        // Generate test samples (1 second of 440Hz sine wave at 44100 Hz)
        let sample_rate = 44100u32;
        let samples: Vec<f32> = (0..sample_rate)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sample_rate as f32).sin())
            .collect();

        AudioRecorder::save_as_wav(&samples, sample_rate, 1, &path).unwrap();

        // Verify file was created
        assert!(path.exists());

        // Read back and verify
        let reader = hound::WavReader::open(&path).unwrap();
        let spec = reader.spec();
        assert_eq!(spec.sample_rate, sample_rate);
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.bits_per_sample, 16);
    }

    #[test]
    fn test_list_input_devices() {
        // This test just verifies the function doesn't panic
        // Actual devices depend on the system
        let result = AudioRecorder::list_input_devices();
        // It's OK if this fails on systems without audio devices
        if let Ok(devices) = result {
            println!("Found {} input devices", devices.len());
            for device in &devices {
                println!("  - {} (default: {})", device.name, device.is_default);
            }
        }
    }
}
