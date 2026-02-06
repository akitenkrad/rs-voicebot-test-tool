//! Audio engine module
//!
//! Handles audio file decoding, playback, waveform generation, and recording.

pub mod decoder;
pub mod player;
pub mod recorder;
pub mod recording;
pub mod silence_detector;
pub mod waveform;

// Re-export key types for convenience.
// These are public API that may not yet be consumed by all downstream modules.
#[allow(unused_imports)]
pub use decoder::{AudioMetadata, DecodedAudio};
#[allow(unused_imports)]
pub use player::{AudioPlayer, PlayerCommand, PlayerStatus};
#[allow(unused_imports)]
pub use recorder::{AudioRecorder, InputDeviceInfo};
#[allow(unused_imports)]
pub use recording::RecordingConfig;
#[allow(unused_imports)]
pub use silence_detector::SilenceDetector;
#[allow(unused_imports)]
pub use waveform::generate_waveform_peaks;
