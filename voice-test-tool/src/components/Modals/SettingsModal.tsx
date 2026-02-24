import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { X } from "lucide-react";
import { useUiStore } from "../../stores/uiStore";
import { useConfig } from "../../hooks/useConfig";
import type { AppConfig, TtsModel, TtsVoice, TtsOutputFormat, RecordingConfig, InputDeviceInfo } from "../../lib/tauri";
import { listInputDevices, getRecordingConfig, updateRecordingConfig } from "../../lib/tauri";
import { Button } from "../ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { Separator } from "../ui/separator";

/** Default recording config */
const defaultRecordingConfig: RecordingConfig = {
  device_name: null,
  sample_rate: 16000,
  channels: 1,
  silence_threshold_db: -40.0,
  silence_duration_ms: 2000,
  max_recording_sec: 60,
};

/** Default config used as fallback when loading fails or config is null */
const defaultConfig: AppConfig = {
  audio: {
    default_sample_rate: 44100,
    default_channels: 1,
    buffer_size: 4096,
    default_playback_speed: 1.0,
    default_volume: 0.8,
  },
  virtual_device: {
    device_name: "VoiceTestTool Virtual Mic",
    auto_create_on_startup: true,
    set_as_default: true,
    cleanup_on_exit: true,
  },
  ui: {
    theme: "system",
    language: "ja",
    window_width: 1200,
    window_height: 800,
    waveform_color: "#4f46e5",
    waveform_progress_color: "#818cf8",
  },
  logging: {
    level: "info",
    output_dir: "./logs",
    max_sessions: 100,
  },
  shortcuts: {
    play_pause: "Space",
    stop: "Escape",
    next: "Ctrl+ArrowRight",
    previous: "Ctrl+ArrowLeft",
    seek_forward: "ArrowRight",
    seek_backward: "ArrowLeft",
  },
  tts: {
    api_key: null,
    base_url: null,
    model: "Gpt4oMiniTts",
    default_voice: "Alloy",
    default_speed: 1.0,
    default_instructions: null,
    output_format: "Wav",
  },
};

export function SettingsModal() {
  const { t } = useTranslation();
  const settingsOpen = useUiStore((state) => state.settingsOpen);
  const setSettingsOpen = useUiStore((state) => state.setSettingsOpen);
  const { config, isLoading, fetchConfig, saveConfig } = useConfig();

  // Local form state
  const [formState, setFormState] = useState<AppConfig>(defaultConfig);

  // Recording config state
  const [recordingConfig, setRecordingConfig] = useState<RecordingConfig>(defaultRecordingConfig);
  const [inputDevices, setInputDevices] = useState<InputDeviceInfo[]>([]);

  // Load config when modal opens
  useEffect(() => {
    if (settingsOpen) {
      fetchConfig().catch(() => {
        // Fallback to default config on error
      });
      // Load recording config and input devices
      getRecordingConfig()
        .then(setRecordingConfig)
        .catch(() => {
          // Use default on error
        });
      listInputDevices()
        .then(setInputDevices)
        .catch(() => {
          // Empty list on error
        });
    }
  }, [settingsOpen, fetchConfig]);

  // Sync fetched config to local form state
  useEffect(() => {
    if (config) {
      setFormState(config);
    }
  }, [config]);

  if (!settingsOpen) {
    return null;
  }

  const handleSave = async () => {
    try {
      await saveConfig(formState);
      // Also save recording config
      await updateRecordingConfig(recordingConfig);
      setSettingsOpen(false);
    } catch {
      // Error is handled by the useConfig hook
    }
  };

  const handleCancel = () => {
    // Discard changes by resetting to the last fetched config
    if (config) {
      setFormState(config);
    }
    setSettingsOpen(false);
  };

  // Helper to update nested audio config
  const updateAudio = (field: keyof AppConfig["audio"], value: number) => {
    setFormState((prev) => ({
      ...prev,
      audio: { ...prev.audio, [field]: value },
    }));
  };

  // Helper to update nested virtual_device config
  const updateDevice = (field: keyof AppConfig["virtual_device"], value: string | boolean) => {
    setFormState((prev) => ({
      ...prev,
      virtual_device: { ...prev.virtual_device, [field]: value },
    }));
  };

  // Helper to update nested ui config
  const updateUi = (field: keyof AppConfig["ui"], value: string | number) => {
    setFormState((prev) => ({
      ...prev,
      ui: { ...prev.ui, [field]: value },
    }));
  };

  // Helper to update nested tts config
  const updateTts = (
    field: keyof AppConfig["tts"],
    value: string | number | null | TtsModel | TtsVoice | TtsOutputFormat
  ) => {
    setFormState((prev) => ({
      ...prev,
      tts: { ...prev.tts, [field]: value },
    }));
  };

  // Common styles for form elements
  const selectClass =
    "w-full rounded-md border border-border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring";
  const inputClass =
    "w-full rounded-md border border-border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring";
  const checkboxClass = "h-4 w-4 rounded border-border text-primary focus:ring-ring";

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-background/80 backdrop-blur-sm"
        onClick={handleCancel}
        role="presentation"
      />

      {/* Modal content */}
      <div className="relative z-50 flex h-[560px] w-[640px] flex-col rounded-lg border border-border bg-background shadow-lg">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-border px-6 py-4">
          <h2 className="text-lg font-semibold">{t("settings.title")}</h2>
          <Button
            variant="ghost"
            size="icon"
            onClick={handleCancel}
            aria-label={t("common.close")}
          >
            <X className="h-4 w-4" />
          </Button>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-hidden p-6">
          {isLoading ? (
            <div className="flex h-full items-center justify-center">
              <p className="text-sm text-muted-foreground">{t("common.loading")}</p>
            </div>
          ) : (
            <Tabs defaultValue="audio" className="flex h-full flex-col">
              <TabsList className="grid w-full grid-cols-6">
                <TabsTrigger value="audio">{t("settings.audio")}</TabsTrigger>
                <TabsTrigger value="device">{t("settings.device")}</TabsTrigger>
                <TabsTrigger value="recording">{t("settings.recording.title")}</TabsTrigger>
                <TabsTrigger value="ui">{t("settings.ui")}</TabsTrigger>
                <TabsTrigger value="tts">{t("settings.tts.title")}</TabsTrigger>
                <TabsTrigger value="shortcuts">{t("settings.shortcuts")}</TabsTrigger>
              </TabsList>

              {/* Audio Tab */}
              <TabsContent value="audio" className="flex-1 overflow-auto">
                <div className="space-y-4 py-4">
                  {/* Sample Rate */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.sampleRate")}</label>
                    <select
                      className={selectClass}
                      value={formState.audio.default_sample_rate}
                      onChange={(e) => updateAudio("default_sample_rate", Number(e.target.value))}
                    >
                      <option value={22050}>22050 Hz</option>
                      <option value={44100}>44100 Hz</option>
                      <option value={48000}>48000 Hz</option>
                    </select>
                  </div>
                  <Separator />

                  {/* Channels */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.channels")}</label>
                    <select
                      className={selectClass}
                      value={formState.audio.default_channels}
                      onChange={(e) => updateAudio("default_channels", Number(e.target.value))}
                    >
                      <option value={1}>{t("settings.mono")} (1)</option>
                      <option value={2}>{t("settings.stereo")} (2)</option>
                    </select>
                  </div>
                  <Separator />

                  {/* Buffer Size */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.bufferSize")}</label>
                    <select
                      className={selectClass}
                      value={formState.audio.buffer_size}
                      onChange={(e) => updateAudio("buffer_size", Number(e.target.value))}
                    >
                      <option value={1024}>1024</option>
                      <option value={2048}>2048</option>
                      <option value={4096}>4096</option>
                      <option value={8192}>8192</option>
                    </select>
                  </div>
                  <Separator />

                  {/* Default Playback Speed */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.playbackSpeed")}</label>
                    <input
                      type="number"
                      className={inputClass}
                      value={formState.audio.default_playback_speed}
                      min={0.5}
                      max={2.0}
                      step={0.1}
                      onChange={(e) => {
                        const val = parseFloat(e.target.value);
                        if (!isNaN(val)) {
                          updateAudio("default_playback_speed", Math.max(0.5, Math.min(2.0, val)));
                        }
                      }}
                    />
                  </div>
                  <Separator />

                  {/* Default Volume */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">
                      {t("settings.defaultVolume")}: {Math.round(formState.audio.default_volume * 100)}%
                    </label>
                    <input
                      type="range"
                      className="w-full accent-primary"
                      value={formState.audio.default_volume}
                      min={0}
                      max={1}
                      step={0.01}
                      onChange={(e) => updateAudio("default_volume", parseFloat(e.target.value))}
                    />
                  </div>
                </div>
              </TabsContent>

              {/* Device Tab */}
              <TabsContent value="device" className="flex-1 overflow-auto">
                <div className="space-y-4 py-4">
                  {/* Device Name */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.deviceName")}</label>
                    <input
                      type="text"
                      className={inputClass}
                      value={formState.virtual_device.device_name}
                      onChange={(e) => updateDevice("device_name", e.target.value)}
                    />
                  </div>
                  <Separator />

                  {/* Auto Create on Startup */}
                  <div className="flex items-center justify-between">
                    <label className="text-sm font-medium">{t("settings.autoCreate")}</label>
                    <label className="relative inline-flex cursor-pointer items-center">
                      <input
                        type="checkbox"
                        className={checkboxClass}
                        checked={formState.virtual_device.auto_create_on_startup}
                        onChange={(e) => updateDevice("auto_create_on_startup", e.target.checked)}
                      />
                      <span className="ml-2 text-sm text-muted-foreground">
                        {formState.virtual_device.auto_create_on_startup
                          ? t("settings.enabled")
                          : t("settings.disabled")}
                      </span>
                    </label>
                  </div>
                  <Separator />

                  {/* Set as Default */}
                  <div className="flex items-center justify-between">
                    <label className="text-sm font-medium">{t("settings.setAsDefault")}</label>
                    <label className="relative inline-flex cursor-pointer items-center">
                      <input
                        type="checkbox"
                        className={checkboxClass}
                        checked={formState.virtual_device.set_as_default}
                        onChange={(e) => updateDevice("set_as_default", e.target.checked)}
                      />
                      <span className="ml-2 text-sm text-muted-foreground">
                        {formState.virtual_device.set_as_default
                          ? t("settings.enabled")
                          : t("settings.disabled")}
                      </span>
                    </label>
                  </div>
                  <Separator />

                  {/* Cleanup on Exit */}
                  <div className="flex items-center justify-between">
                    <label className="text-sm font-medium">{t("settings.cleanupOnExit")}</label>
                    <label className="relative inline-flex cursor-pointer items-center">
                      <input
                        type="checkbox"
                        className={checkboxClass}
                        checked={formState.virtual_device.cleanup_on_exit}
                        onChange={(e) => updateDevice("cleanup_on_exit", e.target.checked)}
                      />
                      <span className="ml-2 text-sm text-muted-foreground">
                        {formState.virtual_device.cleanup_on_exit
                          ? t("settings.enabled")
                          : t("settings.disabled")}
                      </span>
                    </label>
                  </div>
                </div>
              </TabsContent>

              {/* Recording Tab */}
              <TabsContent value="recording" className="flex-1 overflow-auto">
                <div className="space-y-4 py-4">
                  {/* Recording Device */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.recording.device")}</label>
                    <select
                      className={selectClass}
                      value={recordingConfig.device_name ?? ""}
                      onChange={(e) =>
                        setRecordingConfig((prev) => ({
                          ...prev,
                          device_name: e.target.value || null,
                        }))
                      }
                    >
                      <option value="">{t("settings.recording.deviceDefault")}</option>
                      {inputDevices.map((device) => (
                        <option key={device.name} value={device.name}>
                          [{device.device_type === "input" ? t("settings.recording.deviceTypeInput") : t("settings.recording.deviceTypeOutput")}] {device.name} {device.is_default ? "(default)" : ""}
                        </option>
                      ))}
                    </select>
                  </div>
                  <Separator />

                  {/* Silence Threshold */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">
                      {t("settings.recording.silenceThreshold")}: {recordingConfig.silence_threshold_db.toFixed(0)} {t("settings.recording.silenceThresholdUnit")}
                    </label>
                    <input
                      type="range"
                      className="w-full accent-primary"
                      value={recordingConfig.silence_threshold_db}
                      min={-60}
                      max={-20}
                      step={1}
                      onChange={(e) =>
                        setRecordingConfig((prev) => ({
                          ...prev,
                          silence_threshold_db: parseFloat(e.target.value),
                        }))
                      }
                    />
                    <div className="flex justify-between text-xs text-muted-foreground">
                      <span>-60 dB</span>
                      <span>-40 dB</span>
                      <span>-20 dB</span>
                    </div>
                  </div>
                  <Separator />

                  {/* Silence Duration */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">
                      {t("settings.recording.silenceDuration")}: {recordingConfig.silence_duration_ms} {t("settings.recording.silenceDurationUnit")}
                    </label>
                    <input
                      type="range"
                      className="w-full accent-primary"
                      value={recordingConfig.silence_duration_ms}
                      min={500}
                      max={5000}
                      step={100}
                      onChange={(e) =>
                        setRecordingConfig((prev) => ({
                          ...prev,
                          silence_duration_ms: parseInt(e.target.value, 10),
                        }))
                      }
                    />
                    <div className="flex justify-between text-xs text-muted-foreground">
                      <span>500 ms</span>
                      <span>2500 ms</span>
                      <span>5000 ms</span>
                    </div>
                  </div>
                  <Separator />

                  {/* Max Recording Time */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">
                      {t("settings.recording.maxRecordingTime")}: {recordingConfig.max_recording_sec} {t("settings.recording.maxRecordingTimeUnit")}
                    </label>
                    <input
                      type="range"
                      className="w-full accent-primary"
                      value={recordingConfig.max_recording_sec}
                      min={10}
                      max={120}
                      step={5}
                      onChange={(e) =>
                        setRecordingConfig((prev) => ({
                          ...prev,
                          max_recording_sec: parseInt(e.target.value, 10),
                        }))
                      }
                    />
                    <div className="flex justify-between text-xs text-muted-foreground">
                      <span>10s</span>
                      <span>60s</span>
                      <span>120s</span>
                    </div>
                  </div>
                </div>
              </TabsContent>

              {/* UI Tab */}
              <TabsContent value="ui" className="flex-1 overflow-auto">
                <div className="space-y-4 py-4">
                  {/* Theme */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.theme")}</label>
                    <select
                      className={selectClass}
                      value={formState.ui.theme}
                      onChange={(e) => updateUi("theme", e.target.value)}
                    >
                      <option value="light">{t("settings.themeLight")}</option>
                      <option value="dark">{t("settings.themeDark")}</option>
                      <option value="system">{t("settings.themeSystem")}</option>
                    </select>
                  </div>
                  <Separator />

                  {/* Language */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.language")}</label>
                    <select
                      className={selectClass}
                      value={formState.ui.language}
                      onChange={(e) => updateUi("language", e.target.value)}
                    >
                      <option value="ja">日本語</option>
                      <option value="en">English</option>
                    </select>
                  </div>
                  <Separator />

                  {/* Waveform Color */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.waveformColor")}</label>
                    <div className="flex items-center gap-3">
                      <input
                        type="color"
                        className="h-9 w-12 cursor-pointer rounded border border-border bg-background p-0.5"
                        value={formState.ui.waveform_color}
                        onChange={(e) => updateUi("waveform_color", e.target.value)}
                      />
                      <input
                        type="text"
                        className={inputClass + " flex-1"}
                        value={formState.ui.waveform_color}
                        onChange={(e) => updateUi("waveform_color", e.target.value)}
                        placeholder="#4f46e5"
                      />
                    </div>
                  </div>
                  <Separator />

                  {/* Waveform Progress Color */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.waveformProgressColor")}</label>
                    <div className="flex items-center gap-3">
                      <input
                        type="color"
                        className="h-9 w-12 cursor-pointer rounded border border-border bg-background p-0.5"
                        value={formState.ui.waveform_progress_color}
                        onChange={(e) => updateUi("waveform_progress_color", e.target.value)}
                      />
                      <input
                        type="text"
                        className={inputClass + " flex-1"}
                        value={formState.ui.waveform_progress_color}
                        onChange={(e) => updateUi("waveform_progress_color", e.target.value)}
                        placeholder="#818cf8"
                      />
                    </div>
                  </div>
                </div>
              </TabsContent>

              {/* TTS Tab */}
              <TabsContent value="tts" className="flex-1 overflow-auto">
                <div className="space-y-4 py-4">
                  {/* API Key */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.tts.apiKey")}</label>
                    <input
                      type="password"
                      className={inputClass}
                      value={formState.tts.api_key ?? ""}
                      onChange={(e) =>
                        updateTts("api_key", e.target.value || null)
                      }
                      placeholder={t("settings.tts.apiKeyPlaceholder")}
                    />
                  </div>
                  <Separator />

                  {/* Base URL */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.tts.baseUrl")}</label>
                    <input
                      type="text"
                      className={inputClass}
                      value={formState.tts.base_url ?? ""}
                      onChange={(e) =>
                        updateTts("base_url", e.target.value || null)
                      }
                      placeholder={t("settings.tts.baseUrlPlaceholder")}
                    />
                    <p className="text-xs text-muted-foreground">
                      {t("settings.tts.baseUrlHint")}
                    </p>
                  </div>
                  <Separator />

                  {/* Model */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.tts.model")}</label>
                    <select
                      className={selectClass}
                      value={formState.tts.model}
                      onChange={(e) => updateTts("model", e.target.value as TtsModel)}
                    >
                      <option value="Tts1">tts-1</option>
                      <option value="Tts1Hd">tts-1-hd</option>
                      <option value="Gpt4oMiniTts">gpt-4o-mini-tts</option>
                    </select>
                  </div>
                  <Separator />

                  {/* Voice */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.tts.voice")}</label>
                    <select
                      className={selectClass}
                      value={formState.tts.default_voice}
                      onChange={(e) => updateTts("default_voice", e.target.value as TtsVoice)}
                    >
                      <option value="Alloy">Alloy</option>
                      <option value="Ash">Ash</option>
                      <option value="Ballad">Ballad</option>
                      <option value="Coral">Coral</option>
                      <option value="Echo">Echo</option>
                      <option value="Fable">Fable</option>
                      <option value="Nova">Nova</option>
                      <option value="Onyx">Onyx</option>
                      <option value="Sage">Sage</option>
                      <option value="Shimmer">Shimmer</option>
                      <option value="Verse">Verse</option>
                    </select>
                  </div>
                  <Separator />

                  {/* Speed */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">
                      {t("settings.tts.speed")}: {formState.tts.default_speed.toFixed(2)}
                    </label>
                    <input
                      type="range"
                      className="w-full accent-primary"
                      value={formState.tts.default_speed}
                      min={0.25}
                      max={4.0}
                      step={0.05}
                      onChange={(e) => updateTts("default_speed", parseFloat(e.target.value))}
                    />
                    <div className="flex justify-between text-xs text-muted-foreground">
                      <span>0.25</span>
                      <span>1.0</span>
                      <span>4.0</span>
                    </div>
                  </div>
                  <Separator />

                  {/* Output Format */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.tts.outputFormat")}</label>
                    <select
                      className={selectClass}
                      value={formState.tts.output_format}
                      onChange={(e) => updateTts("output_format", e.target.value as TtsOutputFormat)}
                    >
                      <option value="Mp3">MP3</option>
                      <option value="Opus">Opus</option>
                      <option value="Aac">AAC</option>
                      <option value="Flac">FLAC</option>
                      <option value="Wav">WAV</option>
                      <option value="Pcm">PCM</option>
                    </select>
                  </div>
                  <Separator />

                  {/* Instructions */}
                  <div className="space-y-2">
                    <label className="text-sm font-medium">{t("settings.tts.instructions")}</label>
                    <textarea
                      className={inputClass + " min-h-[80px] resize-y"}
                      value={formState.tts.default_instructions ?? ""}
                      onChange={(e) =>
                        updateTts("default_instructions", e.target.value || null)
                      }
                      placeholder={t("settings.tts.instructionsPlaceholder")}
                    />
                  </div>
                </div>
              </TabsContent>

              {/* Shortcuts Tab */}
              <TabsContent value="shortcuts" className="flex-1 overflow-auto">
                <div className="space-y-4 py-4">
                  <p className="text-xs text-muted-foreground">
                    {t("settings.shortcuts")}
                  </p>
                  <div className="overflow-hidden rounded-md border border-border">
                    <table className="w-full text-sm">
                      <thead>
                        <tr className="border-b border-border bg-muted/50">
                          <th className="px-4 py-2 text-left font-medium">{t("settings.action")}</th>
                          <th className="px-4 py-2 text-left font-medium">{t("settings.windowsLinux")}</th>
                          <th className="px-4 py-2 text-left font-medium">{t("settings.macOS")}</th>
                        </tr>
                      </thead>
                      <tbody>
                        <ShortcutRow label={t("settings.shortcutPlayPause")} win="Space" mac="Space" />
                        <ShortcutRow label={t("settings.shortcutStop")} win="Escape" mac="Escape" />
                        <ShortcutRow label={t("settings.shortcutNext")} win="Ctrl + \u2192" mac="Cmd + \u2192" />
                        <ShortcutRow label={t("settings.shortcutPrevious")} win="Ctrl + \u2190" mac="Cmd + \u2190" />
                        <ShortcutRow label={t("settings.shortcutSeekForward")} win="\u2192" mac="\u2192" />
                        <ShortcutRow label={t("settings.shortcutSeekBackward")} win="\u2190" mac="\u2190" />
                        <ShortcutRow label={t("settings.shortcutVolumeUp")} win="\u2191" mac="\u2191" />
                        <ShortcutRow label={t("settings.shortcutVolumeDown")} win="\u2193" mac="\u2193" />
                        <ShortcutRow label={t("settings.shortcutOpenFile")} win="Ctrl + O" mac="Cmd + O" />
                        <ShortcutRow label={t("settings.shortcutOpenSettings")} win="Ctrl + ," mac="Cmd + ," />
                        <ShortcutRow label={t("settings.shortcutRunScenario")} win="F5" mac="F5" />
                        <ShortcutRow label={t("settings.shortcutStopScenario")} win="Shift + F5" mac="Shift + F5" />
                      </tbody>
                    </table>
                  </div>
                </div>
              </TabsContent>
            </Tabs>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-2 border-t border-border px-6 py-4">
          <Button variant="outline" onClick={handleCancel}>
            {t("settings.cancel")}
          </Button>
          <Button onClick={handleSave} disabled={isLoading}>
            {t("settings.save")}
          </Button>
        </div>
      </div>
    </div>
  );
}

/** Shortcut table row component */
function ShortcutRow({ label, win, mac }: { label: string; win: string; mac: string }) {
  return (
    <tr className="border-b border-border last:border-b-0">
      <td className="px-4 py-2 text-muted-foreground">{label}</td>
      <td className="px-4 py-2">
        <kbd className="rounded bg-muted px-1.5 py-0.5 text-xs font-mono">{win}</kbd>
      </td>
      <td className="px-4 py-2">
        <kbd className="rounded bg-muted px-1.5 py-0.5 text-xs font-mono">{mac}</kbd>
      </td>
    </tr>
  );
}
