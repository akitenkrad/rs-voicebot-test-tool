import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  Play,
  Pause,
  Square,
  SkipBack,
  SkipForward,
  Rewind,
  FastForward,
  Volume2,
  VolumeX,
  Repeat,
} from "lucide-react";
import { usePlayback } from "../../hooks/usePlayback";
import { useAudioStore } from "../../stores/audioStore";
import { Button } from "../ui/button";
import { Slider } from "../ui/slider";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "../ui/tooltip";
import { Separator } from "../ui/separator";
import { cn, formatTime } from "../../lib/utils";

const SPEED_OPTIONS = ["0.5", "0.75", "1.0", "1.25", "1.5", "2.0"];

/** Seconds to skip when using rewind/fast-forward buttons */
const SEEK_STEP_SEC = 5;

export function PlaybackControls() {
  const { t } = useTranslation();
  const selectedFileId = useAudioStore((state) => state.selectedFileId);
  const {
    playback,
    playFile,
    pausePlayback,
    stopPlayback,
    seekTo,
    changeSpeed,
    changeVolume,
    toggleLoop,
  } = usePlayback();

  const hasFile = !!selectedFileId;
  const isActive = playback.is_playing || playback.is_paused;

  // -- Handlers --------------------------------------------------------------

  const handlePlayPause = useCallback(async () => {
    if (!selectedFileId) return;

    try {
      if (playback.is_playing) {
        await pausePlayback();
      } else {
        await playFile(selectedFileId);
      }
    } catch (err) {
      console.log("[PlaybackControls] play/pause error:", err);
    }
  }, [selectedFileId, playback.is_playing, playFile, pausePlayback]);

  const handleStop = useCallback(async () => {
    try {
      await stopPlayback();
    } catch (err) {
      console.log("[PlaybackControls] stop error:", err);
    }
  }, [stopPlayback]);

  const handleSeekChange = useCallback(
    async (value: number[]) => {
      try {
        await seekTo(value[0]);
      } catch (err) {
        console.log("[PlaybackControls] seek error:", err);
      }
    },
    [seekTo]
  );

  const handleSeekBackward = useCallback(async () => {
    const newPos = Math.max(0, playback.current_position_sec - SEEK_STEP_SEC);
    try {
      await seekTo(newPos);
    } catch (err) {
      console.log("[PlaybackControls] seek backward error:", err);
    }
  }, [playback.current_position_sec, seekTo]);

  const handleSeekForward = useCallback(async () => {
    const newPos = Math.min(
      playback.total_duration_sec,
      playback.current_position_sec + SEEK_STEP_SEC
    );
    try {
      await seekTo(newPos);
    } catch (err) {
      console.log("[PlaybackControls] seek forward error:", err);
    }
  }, [playback.current_position_sec, playback.total_duration_sec, seekTo]);

  const handleVolumeChange = useCallback(
    async (value: number[]) => {
      try {
        await changeVolume(value[0]);
      } catch (err) {
        console.log("[PlaybackControls] volume error:", err);
      }
    },
    [changeVolume]
  );

  const handleMuteToggle = useCallback(async () => {
    try {
      await changeVolume(playback.volume === 0 ? 0.8 : 0);
    } catch (err) {
      console.log("[PlaybackControls] mute toggle error:", err);
    }
  }, [playback.volume, changeVolume]);

  const handleSpeedChange = useCallback(
    async (value: string) => {
      try {
        await changeSpeed(parseFloat(value));
      } catch (err) {
        console.log("[PlaybackControls] speed change error:", err);
      }
    },
    [changeSpeed]
  );

  const isMuted = playback.volume === 0;

  return (
    <div className="space-y-3 rounded-lg border border-border bg-card p-4">
      {/* Seek bar */}
      <div className="flex items-center gap-3">
        <span className="w-16 text-right text-xs tabular-nums text-muted-foreground">
          {formatTime(playback.current_position_sec)}
        </span>
        <Slider
          value={[playback.current_position_sec]}
          min={0}
          max={playback.total_duration_sec || 1}
          step={0.1}
          onValueChange={handleSeekChange}
          className="flex-1"
          disabled={!hasFile}
        />
        <span className="w-16 text-xs tabular-nums text-muted-foreground">
          {formatTime(playback.total_duration_sec)}
        </span>
      </div>

      {/* Transport controls */}
      <div className="flex items-center justify-center gap-2">
        {/* Previous */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              disabled={!hasFile}
              aria-label={t("playback.previous")}
            >
              <SkipBack className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent><p>{t("playback.previous")}</p></TooltipContent>
        </Tooltip>

        {/* Rewind */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={handleSeekBackward}
              disabled={!isActive}
              aria-label={t("playback.seekBackward")}
            >
              <Rewind className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent><p>{t("playback.seekBackward")}</p></TooltipContent>
        </Tooltip>

        {/* Play/Pause */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="default"
              size="icon"
              className="h-12 w-12 rounded-full"
              onClick={handlePlayPause}
              disabled={!hasFile}
              aria-label={playback.is_playing ? t("playback.pause") : t("playback.play")}
            >
              {playback.is_playing ? (
                <Pause className="h-5 w-5" />
              ) : (
                <Play className="h-5 w-5 ml-0.5" />
              )}
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <p>{playback.is_playing ? t("playback.pause") : t("playback.play")}</p>
          </TooltipContent>
        </Tooltip>

        {/* Stop */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={handleStop}
              disabled={!isActive}
              aria-label={t("playback.stop")}
            >
              <Square className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent><p>{t("playback.stop")}</p></TooltipContent>
        </Tooltip>

        {/* Fast Forward */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={handleSeekForward}
              disabled={!isActive}
              aria-label={t("playback.seekForward")}
            >
              <FastForward className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent><p>{t("playback.seekForward")}</p></TooltipContent>
        </Tooltip>

        {/* Next */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              disabled={!hasFile}
              aria-label={t("playback.next")}
            >
              <SkipForward className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent><p>{t("playback.next")}</p></TooltipContent>
        </Tooltip>
      </div>

      {/* Bottom row: Speed, Volume, Silence, Loop */}
      <div className="flex flex-wrap items-center gap-4">
        {/* Speed selector */}
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">{t("playback.speed")}:</span>
          <Select
            value={playback.speed.toString()}
            onValueChange={handleSpeedChange}
            disabled={!hasFile}
          >
            <SelectTrigger className="h-7 w-20 text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {SPEED_OPTIONS.map((speed) => (
                <SelectItem key={speed} value={speed} className="text-xs">
                  {speed}x
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <Separator orientation="vertical" className="h-5" />

        {/* Volume */}
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={handleMuteToggle}
            aria-label={t("playback.volume")}
          >
            {isMuted ? (
              <VolumeX className="h-4 w-4" />
            ) : (
              <Volume2 className="h-4 w-4" />
            )}
          </Button>
          <Slider
            value={[playback.volume]}
            min={0}
            max={1}
            step={0.01}
            onValueChange={handleVolumeChange}
            className="w-24"
          />
          <span className="w-8 text-xs tabular-nums text-muted-foreground">
            {Math.round(playback.volume * 100)}%
          </span>
        </div>

        <Separator orientation="vertical" className="h-5" />

        {/* Pre-silence / Post-silence */}
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">
            {t("playback.preSilence")}:
          </span>
          <input
            type="number"
            min={0}
            max={10}
            step={0.1}
            defaultValue={0}
            className="h-7 w-16 rounded-md border border-input bg-background px-2 text-xs tabular-nums"
            aria-label={t("playback.preSilence")}
          />
          <span className="text-xs text-muted-foreground">s</span>
        </div>

        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">
            {t("playback.postSilence")}:
          </span>
          <input
            type="number"
            min={0}
            max={10}
            step={0.1}
            defaultValue={0}
            className="h-7 w-16 rounded-md border border-input bg-background px-2 text-xs tabular-nums"
            aria-label={t("playback.postSilence")}
          />
          <span className="text-xs text-muted-foreground">s</span>
        </div>

        <Separator orientation="vertical" className="h-5" />

        {/* Loop toggle */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant={playback.loop_enabled ? "secondary" : "ghost"}
              size="icon"
              className={cn("h-7 w-7", playback.loop_enabled && "text-primary")}
              onClick={toggleLoop}
              disabled={!hasFile}
              aria-label={t("playback.loop")}
            >
              <Repeat className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent><p>{t("playback.loop")}</p></TooltipContent>
        </Tooltip>
      </div>
    </div>
  );
}
