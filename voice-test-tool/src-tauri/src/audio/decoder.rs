//! Audio file decoder
//!
//! Handles decoding of WAV, MP3, FLAC, and OGG audio files.
//! Uses `hound` for WAV files and `symphonia` for all other formats.

use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::AudioError;

/// Metadata extracted from an audio file
#[derive(Debug, Clone)]
pub struct AudioMetadata {
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_sec: f64,
    pub format: String,
    pub total_samples: u64,
}

/// Decoded audio data with metadata
#[derive(Debug, Clone)]
pub struct DecodedAudio {
    pub metadata: AudioMetadata,
    /// Interleaved samples normalized to -1.0..1.0
    pub samples: Vec<f32>,
}

/// Detect the audio format from a file extension
fn detect_format(path: &Path) -> Result<String, AudioError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| AudioError::UnsupportedFormat("no file extension".to_string()))?;

    match ext.as_str() {
        "wav" | "wave" => Ok("wav".to_string()),
        "mp3" => Ok("mp3".to_string()),
        "flac" => Ok("flac".to_string()),
        "ogg" | "oga" => Ok("ogg".to_string()),
        other => Err(AudioError::UnsupportedFormat(other.to_string())),
    }
}

/// Decode a WAV file using hound (simpler and more reliable for WAV)
fn decode_wav(path: &Path) -> Result<DecodedAudio, AudioError> {
    let reader = hound::WavReader::open(path).map_err(|e| {
        if e.to_string().contains("No such file") || e.to_string().contains("not found") {
            AudioError::FileNotFound(path.display().to_string())
        } else {
            AudioError::DecodeError(format!("WAV read error: {}", e))
        }
    })?;

    let spec = reader.spec();
    let sample_rate = spec.sample_rate;
    let channels = spec.channels;
    let total_frames = reader.len() as u64 / channels as u64;
    let duration_sec = total_frames as f64 / sample_rate as f64;

    tracing::info!(
        "Decoding WAV: {}Hz, {} ch, {:.2}s",
        sample_rate,
        channels,
        duration_sec
    );

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .into_samples::<f32>()
            .map(|s| s.map_err(|e| AudioError::DecodeError(format!("WAV sample error: {}", e))))
            .collect::<Result<Vec<f32>, AudioError>>()?,
        hound::SampleFormat::Int => {
            let bits = spec.bits_per_sample;
            let max_val = (1u64 << (bits - 1)) as f32;
            reader
                .into_samples::<i32>()
                .map(|s| {
                    s.map(|v| v as f32 / max_val)
                        .map_err(|e| AudioError::DecodeError(format!("WAV sample error: {}", e)))
                })
                .collect::<Result<Vec<f32>, AudioError>>()?
        }
    };

    let total_samples = samples.len() as u64 / channels as u64;

    Ok(DecodedAudio {
        metadata: AudioMetadata {
            sample_rate,
            channels,
            duration_sec,
            format: "wav".to_string(),
            total_samples,
        },
        samples,
    })
}

/// Decode an audio file using symphonia (MP3, FLAC, OGG, and WAV fallback)
fn decode_with_symphonia(path: &Path, format_hint: &str) -> Result<DecodedAudio, AudioError> {
    let file = std::fs::File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AudioError::FileNotFound(path.display().to_string())
        } else {
            AudioError::DecodeError(format!("File open error: {}", e))
        }
    })?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    hint.with_extension(format_hint);

    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| AudioError::DecodeError(format!("Probe error: {}", e)))?;

    let mut format_reader = probed.format;

    // Find the first audio track
    let track = format_reader
        .tracks()
        .iter()
        .find(|t| {
            t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL
        })
        .ok_or_else(|| AudioError::DecodeError("No audio track found".to_string()))?;

    let track_id = track.id;
    let codec_params = track.codec_params.clone();

    let sample_rate = codec_params
        .sample_rate
        .ok_or_else(|| AudioError::DecodeError("Unknown sample rate".to_string()))?;

    let channels = codec_params
        .channels
        .map(|c| c.count() as u16)
        .unwrap_or(1);

    // Estimate total number of frames for duration calculation
    let n_frames = codec_params.n_frames.unwrap_or(0);
    let duration_sec = if n_frames > 0 {
        n_frames as f64 / sample_rate as f64
    } else {
        0.0
    };

    tracing::info!(
        "Decoding {} with symphonia: {}Hz, {} ch, estimated {:.2}s",
        format_hint,
        sample_rate,
        channels,
        duration_sec
    );

    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &decoder_opts)
        .map_err(|e| AudioError::DecodeError(format!("Codec creation error: {}", e)))?;

    let mut all_samples: Vec<f32> = Vec::new();

    // Decode all packets
    loop {
        let packet = match format_reader.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                // End of stream
                break;
            }
            Err(e) => {
                tracing::warn!("Packet read error (continuing): {}", e);
                break;
            }
        };

        // Skip packets not belonging to our track
        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(symphonia::core::errors::Error::DecodeError(e)) => {
                tracing::warn!("Decode error on packet (skipping): {}", e);
                continue;
            }
            Err(e) => {
                return Err(AudioError::DecodeError(format!("Decode error: {}", e)));
            }
        };

        let spec = *decoded.spec();
        let _num_channels = spec.channels.count();
        let num_frames = decoded.frames();

        if num_frames == 0 {
            continue;
        }

        let mut sample_buf = SampleBuffer::<f32>::new(num_frames as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);

        all_samples.extend_from_slice(sample_buf.samples());

        // Limit for safety: 10 minutes at 48kHz stereo ~= 57M samples
        if all_samples.len() > 60_000_000 {
            tracing::warn!("Audio file exceeds 10 minute limit, truncating");
            break;
        }
    }

    let total_samples = if channels > 0 {
        all_samples.len() as u64 / channels as u64
    } else {
        all_samples.len() as u64
    };

    // Recalculate duration from actual decoded samples
    let actual_duration = total_samples as f64 / sample_rate as f64;

    Ok(DecodedAudio {
        metadata: AudioMetadata {
            sample_rate,
            channels,
            duration_sec: actual_duration,
            format: format_hint.to_string(),
            total_samples,
        },
        samples: all_samples,
    })
}

/// Decode an audio file, auto-detecting the format from the file extension.
///
/// Supports WAV, MP3, FLAC, and OGG formats.
/// Returns decoded audio with interleaved f32 samples normalized to -1.0..1.0.
pub fn decode_file(path: &Path) -> Result<DecodedAudio, AudioError> {
    if !path.exists() {
        return Err(AudioError::FileNotFound(path.display().to_string()));
    }

    let format = detect_format(path)?;

    tracing::info!("Decoding file: {} (format: {})", path.display(), format);

    match format.as_str() {
        "wav" => decode_wav(path),
        "mp3" | "flac" | "ogg" => decode_with_symphonia(path, &format),
        _ => Err(AudioError::UnsupportedFormat(format)),
    }
}

/// Read only metadata from an audio file without full decoding.
///
/// This is faster than `decode_file` when you only need metadata.
pub fn read_metadata(path: &Path) -> Result<AudioMetadata, AudioError> {
    if !path.exists() {
        return Err(AudioError::FileNotFound(path.display().to_string()));
    }

    let format = detect_format(path)?;

    match format.as_str() {
        "wav" => read_wav_metadata(path),
        "mp3" | "flac" | "ogg" => read_symphonia_metadata(path, &format),
        _ => Err(AudioError::UnsupportedFormat(format)),
    }
}

/// Read WAV metadata using hound (no full decode needed)
fn read_wav_metadata(path: &Path) -> Result<AudioMetadata, AudioError> {
    let reader = hound::WavReader::open(path)
        .map_err(|e| AudioError::DecodeError(format!("WAV read error: {}", e)))?;

    let spec = reader.spec();
    let total_frames = reader.len() as u64 / spec.channels as u64;
    let duration_sec = total_frames as f64 / spec.sample_rate as f64;

    Ok(AudioMetadata {
        sample_rate: spec.sample_rate,
        channels: spec.channels,
        duration_sec,
        format: "wav".to_string(),
        total_samples: total_frames,
    })
}

/// Read metadata from MP3/FLAC/OGG using symphonia (without full decode)
fn read_symphonia_metadata(path: &Path, format_hint: &str) -> Result<AudioMetadata, AudioError> {
    let file = std::fs::File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AudioError::FileNotFound(path.display().to_string())
        } else {
            AudioError::DecodeError(format!("File open error: {}", e))
        }
    })?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    hint.with_extension(format_hint);

    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| AudioError::DecodeError(format!("Probe error: {}", e)))?;

    let format_reader = probed.format;

    let track = format_reader
        .tracks()
        .iter()
        .find(|t| {
            t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL
        })
        .ok_or_else(|| AudioError::DecodeError("No audio track found".to_string()))?;

    let codec_params = &track.codec_params;

    let sample_rate = codec_params
        .sample_rate
        .ok_or_else(|| AudioError::DecodeError("Unknown sample rate".to_string()))?;

    let channels = codec_params
        .channels
        .map(|c| c.count() as u16)
        .unwrap_or(1);

    let n_frames = codec_params.n_frames.unwrap_or(0);
    let duration_sec = if n_frames > 0 {
        n_frames as f64 / sample_rate as f64
    } else {
        0.0
    };

    Ok(AudioMetadata {
        sample_rate,
        channels,
        duration_sec,
        format: format_hint.to_string(),
        total_samples: n_frames,
    })
}
