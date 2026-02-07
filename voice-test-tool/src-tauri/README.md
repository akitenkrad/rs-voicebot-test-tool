# voice-test-tool (src-tauri)

Tauri 2.x backend for **Voice Test Tool** — a cross-platform desktop application that automates voice dialogue system (voice chatbot) testing by feeding pre-recorded audio to target systems via virtual microphone devices.

- **Crate name:** `voice-test-tool`
- **Library name:** `voice_test_tool_lib` (staticlib, cdylib, rlib)
- **Edition:** 2021
- **Version:** 0.1.0

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│  React / TypeScript Frontend                                 │
│  (Zustand stores, shadcn/ui, wavesurfer.js)                  │
└──────────────┬───────────────────────────────────────────────┘
               │ Tauri IPC (invoke / events)
┌──────────────▼───────────────────────────────────────────────┐
│  commands/        55 Tauri command handlers                   │
│  (audio, playback, device, playlist, scenario, tts,          │
│   log, config, recording)                                    │
├──────────────────────────────────────────────────────────────┤
│  Domain Modules                                              │
│  ┌─────────┐ ┌─────────┐ ┌──────────┐ ┌─────┐ ┌─────────┐  │
│  │ audio/  │ │ device/ │ │ scenario/│ │ tts/│ │ logging/│  │
│  └─────────┘ └─────────┘ └──────────┘ └─────┘ └─────────┘  │
├──────────────────────────────────────────────────────────────┤
│  state.rs   AppState (Arc<Mutex<T>> shared state)            │
│  config.rs  AppConfig (TOML persistence)                     │
│  error.rs   Error hierarchy (thiserror)                      │
│  types.rs   Frontend-facing data types                       │
└──────────────────────────────────────────────────────────────┘
```

**Key design decisions:**

- **Thread-safe shared state** — `AppState` wraps every field in `Arc<Mutex<T>>`, managed by Tauri as a singleton.
- **Dedicated audio thread** — `AudioPlayer` runs on its own OS thread, receiving commands via crossbeam channels. This avoids latency issues from running real-time audio on the tokio executor.
- **Async runtime** — tokio (`full` features) powers background tasks: scenario execution, TTS test runs, and recording.
- **Platform abstraction** — The `VirtualDeviceManager` trait provides a uniform interface over Linux (PulseAudio), macOS (BlackHole), and Windows (VB-Cable) device backends.

## Module Structure

```
src/
├── main.rs                     Entry point
├── lib.rs                      Library exports, Tauri app builder
├── state.rs                    AppState with all shared state fields
├── config.rs                   AppConfig (TOML), platform-specific paths
├── error.rs                    Error hierarchy (AudioError, DeviceError, etc.)
├── types.rs                    Shared frontend-facing data types
│
├── audio/
│   ├── mod.rs                  Module exports
│   ├── decoder.rs              Multi-format decoding (symphonia + hound)
│   ├── player.rs               Playback engine (rodio, crossbeam channels)
│   ├── recorder.rs             Audio recording (cpal)
│   ├── recording.rs            RecordingConfig
│   ├── silence_detector.rs     RMS-based silence detection
│   └── waveform.rs             Peak generation for visualization
│
├── device/
│   ├── mod.rs                  VirtualDeviceManager trait, factory fn
│   ├── linux.rs                LinuxDeviceManager (PulseAudio null-sink)
│   ├── macos.rs                MacOSDeviceManager (BlackHole/Soundflower)
│   └── windows.rs              WindowsDeviceManager (VB-Cable/VoiceMeeter)
│
├── scenario/
│   ├── mod.rs                  Module exports
│   ├── parser.rs               JSON/YAML scenario file parsing
│   └── executor.rs             Sequential execution with pause/resume/abort
│
├── tts/
│   ├── mod.rs                  Module exports
│   ├── config.rs               TtsConfig, TtsModel, TtsVoice, TtsOutputFormat
│   ├── types.rs                TtsTestSession, TtsTestResult, TtsTestStatus
│   ├── csv_parser.rs           Single-column CSV test case parsing
│   ├── generator.rs            OpenAI TTS audio generation
│   ├── runner.rs               TtsTestRunner with recording integration
│   └── exporter.rs             ZIP export for TTS test sessions
│
├── logging/
│   ├── mod.rs                  Module exports
│   └── reporter.rs             TestLogger, TestSession, report export
│
└── commands/
    ├── mod.rs                  Module exports
    ├── audio.rs                File loading, waveform data (4 commands)
    ├── playback.rs             Play, pause, stop, seek, volume (7 commands)
    ├── device.rs               Virtual device CRUD (5 commands)
    ├── playlist.rs             Playlist CRUD and playback (11 commands)
    ├── scenario.rs             Scenario load/save/execute (9 commands)
    ├── tts.rs                  TTS generation and test execution (9 commands)
    ├── log.rs                  Session logs and report export (4 commands)
    ├── config.rs               Config read/write (2 commands)
    └── recording.rs            Input device listing and config (3 commands)
```

## Module Details

### audio/

Multi-format audio decoding, playback, recording, and analysis.

- **decoder.rs** — `decode_file()` and `read_metadata()`. WAV files use the optimized `hound` path; MP3/FLAC/OGG go through `symphonia`. Output: interleaved f32 samples normalized to \[-1.0, 1.0\]. Hard limit: 10 min at 48 kHz stereo (~57M samples).
- **player.rs** — `AudioPlayer` spawns a dedicated thread. Commands (`Play`, `Pause`, `Stop`, `Seek`, `SetVolume`, `SetSpeed`, `Load`, `Shutdown`) are sent via crossbeam channel. Speed range: 0.5–2.0x. Volume range: 0.0–1.0.
- **recorder.rs** — `AudioRecorder` uses `cpal` for cross-platform capture. Supports both input devices (microphones) and output devices (loopback). Integrates with `SilenceDetector` for automatic stop.
- **recording.rs** — `RecordingConfig`: device name, sample rate (default 16 kHz), channels (default mono), silence threshold (-40 dB), silence duration (2 s), max recording time (60 s).
- **silence_detector.rs** — `SilenceDetector` monitors RMS amplitude against a configurable dB threshold and duration.
- **waveform.rs** — `generate_waveform_peaks()` downsamples audio into RMS peak data at a given resolution for frontend visualization.

### device/

Platform-specific virtual audio device management behind a common trait.

```rust
pub trait VirtualDeviceManager: Send + Sync {
    fn create_device(&mut self, name: &str) -> Result<DeviceInfo, DeviceError>;
    fn destroy_device(&mut self) -> Result<(), DeviceError>;
    fn list_devices(&self) -> Result<Vec<DeviceInfo>, DeviceError>;
    fn detect_virtual_device(&self) -> Result<Option<DeviceInfo>, DeviceError>;
    fn get_status(&self) -> Result<DeviceStatus, DeviceError>;
    fn active_device_name(&self) -> Option<String>;
}
```

| Platform | Implementation | Mechanism | Auto-create? |
|----------|---------------|-----------|:------------:|
| Linux | `LinuxDeviceManager` | PulseAudio null-sink via `pactl load-module` / `pactl unload-module` | Yes |
| macOS | `MacOSDeviceManager` | Detects BlackHole, Soundflower, Background Music, Loopback via `cpal` enumeration | No |
| Windows | `WindowsDeviceManager` | Detects CABLE, VB-Audio, VoiceMeeter, Virtual Cable via `cpal` enumeration | No |

### scenario/

JSON/YAML scenario parsing and sequential execution.

- **parser.rs** — Reads scenario files, resolves relative audio paths, converts to runtime types.
- **executor.rs** — `run_scenario()` plays turns sequentially with configurable delays (`delay_before_ms`, `delay_after_ms`). Supports pause/resume/abort via `AtomicBool` flags. Integrates with `TestLogger` for event tracking.

Key types:
- `ScenarioExecStatus` — `Running`, `Paused`, `Completed`, `Failed(String)`, `Aborted`
- `ScenarioExecutionState` — Tracks session ID, progress, and control flags.

### tts/

OpenAI TTS integration for automated voice generation and testing.

- **config.rs** — `TtsConfig` with model selection (`Tts1`, `Tts1Hd`, `Gpt4oMiniTts`), 11 voice options, output format (MP3/Opus/AAC/FLAC/WAV/PCM), speed, and optional instructions.
- **csv_parser.rs** — Parses single-column CSV files into `TtsTestCase` entries with auto-generated sequential IDs.
- **generator.rs** — Wraps `openai-tools` crate to convert text to audio bytes.
- **runner.rs** — `TtsTestRunner` manages sequential execution: generates TTS audio, plays it, records system response, supports pause/resume/abort, and emits Tauri events for progress.
- **exporter.rs** — Exports test sessions as ZIP archives containing metadata JSON and audio files.
- **types.rs** — `TtsTestSession`, `TtsTestResult`, `TtsTestStatus` (tagged enum: `Pending`, `Running`, `Paused`, `Completed`, `Aborted`, `Failed`).

### logging/

Test session logging and multi-format report export.

- **reporter.rs** — `TestLogger` stores sessions and events in memory. Supports `create_session()`, `log_event()`, `end_session()`. Exports to JSON, CSV, and HTML via `export_json()`, `export_csv()`, `export_html()`.

### commands/

55 Tauri command handlers organized by domain. Each function takes `State<AppState>` and returns `Result<T, String>`.

### state.rs

`AppState` holds all mutable application state:

```rust
pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub audio_files: Arc<Mutex<Vec<AudioFileInfo>>>,
    pub playback_state: Arc<Mutex<PlaybackState>>,
    pub active_device: Arc<Mutex<Option<DeviceInfo>>>,
    pub device_manager: Arc<Mutex<Box<dyn VirtualDeviceManager>>>,
    pub scenarios: Arc<Mutex<Vec<ScenarioInfo>>>,
    pub playlists: Arc<Mutex<Vec<PlaylistInfo>>>,
    pub active_sessions: Arc<Mutex<Vec<String>>>,
    pub player: Arc<Mutex<Option<AudioPlayer>>>,
    pub decoded_cache: Arc<Mutex<HashMap<String, DecodedAudio>>>,
    pub playlist_playback: Arc<Mutex<Option<PlaylistPlaybackState>>>,
    pub test_logger: Arc<Mutex<TestLogger>>,
    pub scenario_executions: Arc<Mutex<HashMap<String, ScenarioExecutionState>>>,
    pub recording_config: Arc<Mutex<RecordingConfig>>,
    pub audio_recorder: Arc<Mutex<Option<AudioRecorder>>>,
    pub tts_sessions: Arc<Mutex<HashMap<String, TtsTestRunner>>>,
}
```

### error.rs

Hierarchical error types using `thiserror`:

- `AudioError` — `FileNotFound`, `UnsupportedFormat`, `PlaybackFailed`, `DecodeError`
- `DeviceError` — `CreationFailed`, `NotFound`, `PlatformUnsupported`
- `ScenarioError` — `ParseError`, `ExecutionFailed`, `InvalidScenario`
- `ConfigError` — `LoadFailed`, `SaveFailed`, `ValidationError`
- `AppError` — Wraps all of the above plus `Internal(String)`

### config.rs

`AppConfig` with nested sections: `AudioConfig`, `VirtualDeviceConfig`, `UiConfig`, `LoggingConfig`, `ShortcutConfig`, `TtsConfig`. Persisted as TOML in platform-specific config directories (via `directories` crate).

## Tauri Commands

### Audio (4)

| Command | Description |
|---------|-------------|
| `load_audio_file` | Load and decode an audio file |
| `list_audio_files` | List all loaded audio files |
| `remove_audio_file` | Remove a loaded audio file |
| `get_waveform_data` | Generate waveform peaks at given resolution |

### Playback (7)

| Command | Description |
|---------|-------------|
| `play` | Start playback of a loaded file |
| `pause` | Pause current playback |
| `stop` | Stop current playback |
| `seek` | Seek to position (seconds) |
| `set_playback_speed` | Set speed (0.5–2.0x) |
| `set_volume` | Set volume (0.0–1.0) |
| `get_playback_state` | Query current playback state |

### Device (5)

| Command | Description |
|---------|-------------|
| `create_virtual_device` | Create a virtual audio device (Linux only) |
| `destroy_virtual_device` | Destroy the active virtual device |
| `list_audio_devices` | List all audio devices |
| `set_default_input_device` | Set a device as default input |
| `get_device_status` | Query virtual device status |

### Playlist (11)

| Command | Description |
|---------|-------------|
| `create_playlist` | Create a new playlist |
| `add_to_playlist` | Add audio file with silence config |
| `list_playlists` | List all playlists |
| `get_playlist` | Get playlist details |
| `delete_playlist` | Delete a playlist |
| `remove_from_playlist` | Remove an item from playlist |
| `reorder_playlist` | Reorder playlist items |
| `update_playlist_item_silence` | Update pre/post silence |
| `play_playlist` | Start sequential playback (with optional loop) |
| `get_playlist_playback_status` | Query playlist playback progress |
| `stop_playlist` | Stop playlist playback |

### Scenario (9)

| Command | Description |
|---------|-------------|
| `load_scenario` | Load scenario from JSON/YAML file |
| `save_scenario` | Save scenario to file |
| `execute_scenario` | Start scenario execution (returns session ID) |
| `pause_scenario` | Pause running scenario |
| `resume_scenario` | Resume paused scenario |
| `abort_scenario` | Abort running scenario |
| `list_scenarios` | List all loaded scenarios |
| `delete_scenario` | Delete a scenario |
| `get_scenario_execution_status` | Query execution progress |

### TTS (9)

| Command | Description |
|---------|-------------|
| `load_tts_csv` | Parse TTS test cases from CSV |
| `preview_tts` | Generate TTS preview audio |
| `start_tts_test` | Start TTS test session (returns session ID) |
| `pause_tts_test` | Pause running TTS test |
| `resume_tts_test` | Resume paused TTS test |
| `abort_tts_test` | Abort running TTS test |
| `get_tts_test_status` | Query TTS test progress |
| `get_tts_test_session` | Get full TTS session details |
| `export_tts_test` | Export TTS session as ZIP |

### Logging (4)

| Command | Description |
|---------|-------------|
| `get_session_logs` | Get log entries for a session |
| `export_report` | Export report (JSON/CSV/HTML) |
| `list_test_sessions` | List all test sessions |
| `get_test_session` | Get session details |

### Config (2)

| Command | Description |
|---------|-------------|
| `get_config` | Read current configuration |
| `update_config` | Write updated configuration |

### Recording (3)

| Command | Description |
|---------|-------------|
| `list_input_devices` | List available input devices |
| `get_recording_config` | Get recording settings |
| `update_recording_config` | Update recording settings |

## Key Dependencies

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `tauri` | 2.10 | Application framework (IPC, window, lifecycle) |
| `tauri-plugin-shell` | 2.3 | Shell command execution |
| `tauri-plugin-dialog` | 2.6 | Native file dialogs |
| `tauri-plugin-fs` | 2.4 | File system access |
| `rodio` | 0.19 | Audio playback |
| `cpal` | 0.15 | Cross-platform audio I/O (recording) |
| `symphonia` | 0.5 | Multi-format audio decoding (MP3, FLAC, OGG, WAV) |
| `hound` | 3.5 | Optimized WAV file I/O |
| `rubato` | 0.15 | Sample rate conversion |
| `dasp` | 0.11 | Digital audio signal processing |
| `tokio` | 1.49 | Async runtime |
| `crossbeam-channel` | 0.5 | Lock-free MPMC channels (audio thread) |
| `parking_lot` | 0.12 | Efficient synchronization primitives |
| `serde` | 1.0 | Serialization framework |
| `serde_json` | 1.0 | JSON serialization |
| `serde_yaml` | 0.9 | YAML scenario parsing |
| `toml` | 0.8 | Config file format |
| `tracing` | 0.1 | Structured logging |
| `tracing-subscriber` | 0.3 | Log output and filtering |
| `tracing-appender` | 0.2 | Log file output |
| `thiserror` | 2.0 | Derive macro for error types |
| `anyhow` | 1.0 | Flexible error handling |
| `uuid` | 1.20 | Unique ID generation (v4) |
| `chrono` | 0.4 | Date/time handling |
| `directories` | 5.0 | Platform-specific config/data paths |
| `notify` | 6.1 | File system event watching |
| `tempfile` | 3.24 | Temporary file management |
| `openai-tools` | local | OpenAI TTS API client |
| `csv` | 1.3 | CSV parsing for TTS test cases |
| `zip` | 2.2 | ZIP archive creation for exports |

### Platform-Specific Dependencies

| Platform | Dependency | Version | Purpose |
|----------|-----------|---------|---------|
| Linux | `libpulse-binding` | 2.30 | PulseAudio API bindings |
| Linux | `libpulse-simple-binding` | 2.29 | PulseAudio simple API |
| macOS | `coreaudio-rs` | 0.11 | CoreAudio bindings |
| Windows | `windows` | 0.58 | Win32 audio APIs |
| Windows | `wasapi` | 0.14 | WASAPI audio interface |

### Dev Dependencies

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `rstest` | 0.23 | Parameterized test fixtures |
| `mockall` | 0.13 | Mock generation for trait testing |

## Platform Requirements

| OS | Virtual Device | Detection Method | Auto-Create | Prerequisites |
|----|---------------|-----------------|:-----------:|---------------|
| Linux | PulseAudio null-sink | `pactl list sinks short` | Yes | PulseAudio installed |
| macOS | BlackHole / Soundflower / Background Music / Loopback | `cpal` device enumeration | No | Driver pre-installed by user |
| Windows | VB-Cable / VoiceMeeter | `cpal` device enumeration | No | Driver pre-installed by user |

## Configuration

Configuration is stored as TOML in platform-specific directories (resolved by the `directories` crate):

| OS | Typical Path |
|----|-------------|
| Linux | `~/.config/VoiceTestTool/config.toml` |
| macOS | `~/Library/Application Support/VoiceTestTool/config.toml` |
| Windows | `%APPDATA%\VoiceTestTool\config.toml` |

### AppConfig Structure

```toml
[audio]
default_sample_rate = 44100
default_channels = 2
buffer_size = 4096
default_playback_speed = 1.0
default_volume = 1.0

[virtual_device]
device_name = "VoiceTestTool"
auto_create_on_startup = false
set_as_default = false
cleanup_on_exit = true

[ui]
theme = "system"
language = "ja"
window_width = 1280
window_height = 720
waveform_color = "#4a9eff"
waveform_progress_color = "#ff6b6b"

[logging]
level = "info"
output_dir = "logs"
max_sessions = 100

[shortcuts]
play_pause = "Space"
stop = "Escape"
next = "Right"
previous = "Left"
seek_forward = "Shift+Right"
seek_backward = "Shift+Left"

[tts]
model = "Tts1"
default_voice = "Alloy"
default_speed = 1.0
output_format = "Mp3"
```

## Build & Development

### Prerequisites

- **Rust toolchain** — stable (edition 2021)
- **Node.js** — for the frontend build
- **Platform libraries:**
  - Linux: `libpulse-dev`, `libasound2-dev`
  - macOS: Xcode command-line tools (CoreAudio included)
  - Windows: Visual Studio build tools

### Commands

```bash
# Development (Vite hot-reload + Rust backend)
npm run tauri dev

# Production build (platform-specific installer)
npm run tauri build

# Rust-only build
cd src-tauri && cargo build

# Rust-only tests
cd src-tauri && cargo test

# Clear Rust build cache
cd src-tauri && cargo clean
```

### Feature Flags

| Flag | Default | Description |
|------|:-------:|-------------|
| `pipewire` | off | Enable PipeWire support on Linux (experimental) |

```bash
# Build with PipeWire support
cd src-tauri && cargo build --features pipewire
```
