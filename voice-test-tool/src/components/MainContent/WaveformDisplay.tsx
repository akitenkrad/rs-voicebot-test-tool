import { useEffect, useRef, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Music, Loader2 } from "lucide-react";
import { useAudioStore } from "../../stores/audioStore";
import { usePlaybackStore } from "../../stores/playbackStore";
import { useAudio } from "../../hooks/useAudio";
import { usePlayback } from "../../hooks/usePlayback";
import { useWaveSurfer } from "../../hooks/useWaveSurfer";
import { Badge } from "../ui/badge";
import { formatTime } from "../../lib/utils";

/** Default resolution (number of peaks) when fetching waveform data */
const DEFAULT_RESOLUTION = 1000;

export function WaveformDisplay() {
  const { t } = useTranslation();
  const files = useAudioStore((state) => state.files);
  const selectedFileId = useAudioStore((state) => state.selectedFileId);
  const playback = usePlaybackStore((state) => state.playback);
  const { getWaveform } = useAudio();
  const { seekTo: backendSeekTo } = usePlayback();

  const waveformContainerRef = useRef<HTMLDivElement>(null);
  const [isLoadingWaveform, setIsLoadingWaveform] = useState(false);
  const [waveformError, setWaveformError] = useState<string | null>(null);
  const [hasPeaks, setHasPeaks] = useState(false);

  const selectedFile = files.find((f) => f.id === selectedFileId);

  // ----- WaveSurfer hook -----

  const handleSeek = useCallback(
    (positionSec: number) => {
      backendSeekTo(positionSec).catch((err: unknown) => {
        console.log("[WaveformDisplay] seek error:", err);
      });
    },
    [backendSeekTo]
  );

  const { loadPeaks, seekTo: wsSyncPosition, isReady } = useWaveSurfer({
    container: waveformContainerRef,
    onSeek: handleSeek,
  });

  // ----- Fetch waveform peaks when selected file changes -----

  useEffect(() => {
    if (!selectedFileId || !selectedFile) {
      setHasPeaks(false);
      setWaveformError(null);
      return;
    }

    let cancelled = false;

    const fetchWaveform = async () => {
      setIsLoadingWaveform(true);
      setWaveformError(null);
      setHasPeaks(false);
      try {
        const data = await getWaveform(selectedFileId, DEFAULT_RESOLUTION);
        if (!cancelled && isReady) {
          loadPeaks(data.peaks, selectedFile.duration_sec);
          setHasPeaks(true);
        }
      } catch (err) {
        if (!cancelled) {
          const message = err instanceof Error ? err.message : String(err);
          setWaveformError(message);
          setHasPeaks(false);
          console.log("[WaveformDisplay] error loading waveform:", message);
        }
      } finally {
        if (!cancelled) {
          setIsLoadingWaveform(false);
        }
      }
    };

    fetchWaveform();

    return () => {
      cancelled = true;
    };
  }, [selectedFileId, selectedFile, getWaveform, isReady, loadPeaks]);

  // ----- Sync playback position from backend -> wavesurfer -----

  useEffect(() => {
    if (!selectedFile || !isReady || !hasPeaks) return;
    wsSyncPosition(playback.current_position_sec, selectedFile.duration_sec);
  }, [
    playback.current_position_sec,
    selectedFile,
    isReady,
    hasPeaks,
    wsSyncPosition,
  ]);

  // ----- Render -----

  // No file selected -- placeholder
  if (!selectedFile) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center rounded-lg border border-dashed border-border bg-muted/30 p-8">
        <Music className="mb-4 h-12 w-12 text-muted-foreground/50" />
        <p className="text-sm text-muted-foreground">{t("waveform.noFile")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-1 flex-col rounded-lg border border-border bg-card">
      {/* File info bar */}
      <div className="flex items-center gap-3 border-b border-border px-4 py-2">
        <span className="text-sm font-medium text-foreground">
          {selectedFile.name}
        </span>
        <Badge variant="secondary" className="text-[10px]">
          {selectedFile.format.toUpperCase()}
        </Badge>
        <span className="text-xs text-muted-foreground">
          {selectedFile.sample_rate.toLocaleString()} Hz
        </span>
        <span className="text-xs text-muted-foreground">
          {selectedFile.channels}ch
        </span>
        <span className="text-xs text-muted-foreground">
          {formatTime(selectedFile.duration_sec)}
        </span>
      </div>

      {/* Waveform area */}
      <div className="relative flex flex-1 items-center justify-center overflow-hidden p-4">
        {/* WaveSurfer container -- always mounted so the ref stays stable */}
        <div
          ref={waveformContainerRef}
          className="absolute inset-0 m-4"
          style={{
            // Hide the canvas until we have real peaks to show
            visibility: hasPeaks && !isLoadingWaveform ? "visible" : "hidden",
          }}
        />

        {/* Overlay states rendered on top of the (hidden) wavesurfer canvas */}
        {isLoadingWaveform && (
          <div className="z-10 flex flex-col items-center gap-2 text-muted-foreground">
            <Loader2 className="h-8 w-8 animate-spin" />
            <span className="text-xs">{t("common.loading")}</span>
          </div>
        )}

        {!isLoadingWaveform && waveformError && (
          <div className="z-10 text-center text-xs text-destructive">
            <p>{t("waveform.error")}</p>
            <p className="mt-1 text-muted-foreground">{waveformError}</p>
          </div>
        )}

        {!isLoadingWaveform && !waveformError && !hasPeaks && (
          <div className="z-10 flex flex-col items-center gap-2 text-muted-foreground">
            <Music className="h-8 w-8 text-muted-foreground/50" />
            <span className="text-xs">{t("common.loading")}</span>
          </div>
        )}
      </div>
    </div>
  );
}
