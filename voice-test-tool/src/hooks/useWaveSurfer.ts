import { useEffect, useRef, useCallback, useState } from "react";
import WaveSurfer from "wavesurfer.js";

/** Helper to read a CSS custom property as an HSL color string */
function getCssColor(varName: string): string {
  const value = getComputedStyle(document.documentElement)
    .getPropertyValue(varName)
    .trim();
  return value ? `hsl(${value})` : "#888";
}

export interface UseWaveSurferOptions {
  /** Ref to the container element where WaveSurfer should render */
  container: React.RefObject<HTMLDivElement | null>;
  /** Callback when the user clicks/drags on the waveform to seek */
  onSeek?: (positionSec: number) => void;
}

export interface UseWaveSurferReturn {
  /** Load peak data into the waveform display */
  loadPeaks: (peaks: number[], durationSec: number) => void;
  /** Programmatically update the playback cursor position (0..duration in seconds) */
  seekTo: (positionSec: number, durationSec: number) => void;
  /** Whether the WaveSurfer instance has been created and is ready */
  isReady: boolean;
  /** Destroy the current instance (also called automatically on unmount) */
  destroy: () => void;
}

/**
 * Reusable hook that manages a WaveSurfer instance for visualization only.
 *
 * Audio playback is handled entirely by the Rust backend -- this hook only
 * renders a waveform from pre-computed peak data and forwards seek interactions.
 */
export function useWaveSurfer(
  options: UseWaveSurferOptions
): UseWaveSurferReturn {
  const { container, onSeek } = options;

  const wsRef = useRef<WaveSurfer | null>(null);
  const [isReady, setIsReady] = useState(false);

  /**
   * Flag to suppress seek callbacks that originate from programmatic
   * position updates (i.e. syncing from the backend) rather than user
   * interactions on the waveform.
   */
  const isProgrammaticSeekRef = useRef(false);

  /** Keep the latest onSeek in a ref so event handlers always see the current version */
  const onSeekRef = useRef(onSeek);
  useEffect(() => {
    onSeekRef.current = onSeek;
  }, [onSeek]);

  /** Store duration so click handler can compute absolute position */
  const durationRef = useRef(0);

  // -----------------------------------------------------------------------
  // Create / destroy the WaveSurfer instance when the container is available
  // -----------------------------------------------------------------------
  useEffect(() => {
    const el = container.current;
    if (!el) return;

    const ws = WaveSurfer.create({
      container: el,
      waveColor: getCssColor("--waveform"),
      progressColor: getCssColor("--waveform-progress"),
      cursorColor: getCssColor("--primary"),
      barWidth: 3,
      barGap: 2,
      barRadius: 2,
      height: "auto",
      normalize: true,
      interact: true,
      dragToSeek: true,
      hideScrollbar: true,
      // We never load an actual audio URL -- playback is via Rust backend
    });

    // --- User interaction events ---

    ws.on("interaction", (newTime: number) => {
      if (isProgrammaticSeekRef.current) return;
      onSeekRef.current?.(newTime);
    });

    wsRef.current = ws;
    setIsReady(true);

    return () => {
      ws.destroy();
      wsRef.current = null;
      setIsReady(false);
    };
    // Only re-create when the container element itself changes
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [container]);

  // -----------------------------------------------------------------------
  // Observe theme changes (dark class toggling on <html>) and update colors
  // -----------------------------------------------------------------------
  useEffect(() => {
    const observer = new MutationObserver(() => {
      wsRef.current?.setOptions({
        waveColor: getCssColor("--waveform"),
        progressColor: getCssColor("--waveform-progress"),
        cursorColor: getCssColor("--primary"),
      });
    });

    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });

    return () => observer.disconnect();
  }, []);

  // -----------------------------------------------------------------------
  // Handle container resize so the waveform stays responsive
  // -----------------------------------------------------------------------
  useEffect(() => {
    const el = container.current;
    if (!el) return;

    const ro = new ResizeObserver(() => {
      // setOptions with the same values triggers a re-render at the new size
      wsRef.current?.setOptions({ height: "auto" });
    });

    ro.observe(el);
    return () => ro.disconnect();
  }, [container]);

  // -----------------------------------------------------------------------
  // Public API
  // -----------------------------------------------------------------------

  const loadPeaks = useCallback((peaks: number[], durationSec: number) => {
    const ws = wsRef.current;
    if (!ws) return;
    durationRef.current = durationSec;

    // load(url, peaks, duration) -- empty string URL since we don't load audio
    ws.load("", [peaks], durationSec);
  }, []);

  const seekTo = useCallback((positionSec: number, durationSec: number) => {
    const ws = wsRef.current;
    if (!ws || durationSec <= 0) return;

    const ratio = Math.max(0, Math.min(1, positionSec / durationSec));
    isProgrammaticSeekRef.current = true;
    ws.seekTo(ratio);
    isProgrammaticSeekRef.current = false;
  }, []);

  const destroy = useCallback(() => {
    wsRef.current?.destroy();
    wsRef.current = null;
    setIsReady(false);
  }, []);

  return { loadPeaks, seekTo, isReady, destroy };
}
