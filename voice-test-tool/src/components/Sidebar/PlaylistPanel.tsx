import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Plus,
  GripVertical,
  Trash2,
  Play,
  Square,
  Repeat,
  FilePlus,
} from "lucide-react";
import { Button } from "../ui/button";
import { ScrollArea } from "../ui/scroll-area";
import { Badge } from "../ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import { usePlaylist } from "../../hooks/usePlaylist";
import { useAudioStore } from "../../stores/audioStore";
import type { PlaylistItemInfo } from "../../types/audio";

/** Polling interval for playlist playback status (ms) */
const PLAYBACK_POLL_INTERVAL = 500;

export function PlaylistPanel() {
  const { t } = useTranslation();
  const {
    playlists,
    selectedPlaylist,
    selectedPlaylistId,
    playbackStatus,
    isLoading,
    error,
    selectedFileId,
    fetchPlaylists,
    createNewPlaylist,
    deletePlaylist,
    addFileToPlaylist,
    removeFileFromPlaylist,
    reorderItems,
    updateSilence,
    playSelectedPlaylist,
    stopPlaylistPlayback,
    refreshPlaybackStatus,
    setSelectedPlaylistId,
  } = usePlaylist();

  const audioFiles = useAudioStore((s) => s.files);

  const [loopMode, setLoopMode] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [newPlaylistName, setNewPlaylistName] = useState("");
  const newPlaylistInputRef = useRef<HTMLInputElement>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Drag state
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);

  // Load playlists on mount
  useEffect(() => {
    fetchPlaylists();
  }, [fetchPlaylists]);

  // Poll playback status while playing
  useEffect(() => {
    if (playbackStatus?.is_playing) {
      if (pollRef.current === null) {
        pollRef.current = setInterval(() => {
          refreshPlaybackStatus();
        }, PLAYBACK_POLL_INTERVAL);
      }
    } else {
      if (pollRef.current !== null) {
        clearInterval(pollRef.current);
        pollRef.current = null;
      }
    }
    return () => {
      if (pollRef.current !== null) {
        clearInterval(pollRef.current);
        pollRef.current = null;
      }
    };
  }, [playbackStatus?.is_playing, refreshPlaybackStatus]);

  // Focus input when creating
  useEffect(() => {
    if (isCreating) {
      newPlaylistInputRef.current?.focus();
    }
  }, [isCreating]);

  // -- Handlers ---------------------------------------------------------------

  const handleCreatePlaylist = useCallback(async () => {
    const name = newPlaylistName.trim();
    if (!name) return;
    try {
      await createNewPlaylist(name);
      setNewPlaylistName("");
      setIsCreating(false);
    } catch {
      // Error is set in the hook
    }
  }, [newPlaylistName, createNewPlaylist]);

  const handleDeletePlaylist = useCallback(async () => {
    if (!selectedPlaylistId) return;
    if (!window.confirm(t("playlist.confirmDelete"))) return;
    try {
      await deletePlaylist(selectedPlaylistId);
    } catch {
      // Error is set in the hook
    }
  }, [selectedPlaylistId, deletePlaylist, t]);

  const handlePlay = useCallback(async () => {
    await playSelectedPlaylist(loopMode);
    // Start polling immediately
    refreshPlaybackStatus();
  }, [playSelectedPlaylist, loopMode, refreshPlaybackStatus]);

  const handleStop = useCallback(async () => {
    await stopPlaylistPlayback();
  }, [stopPlaylistPlayback]);

  const handleAddFile = useCallback(async () => {
    await addFileToPlaylist(0, 0);
  }, [addFileToPlaylist]);

  const handleRemoveItem = useCallback(
    async (itemId: string) => {
      await removeFileFromPlaylist(itemId);
    },
    [removeFileFromPlaylist]
  );

  const handleSilenceChange = useCallback(
    async (item: PlaylistItemInfo, field: "pre" | "post", value: number) => {
      const pre = field === "pre" ? value : item.pre_silence_sec;
      const post = field === "post" ? value : item.post_silence_sec;
      await updateSilence(item.id, pre, post);
    },
    [updateSilence]
  );

  // -- Drag and drop ----------------------------------------------------------

  const handleDragStart = useCallback(
    (e: React.DragEvent, index: number) => {
      setDragIndex(index);
      e.dataTransfer.effectAllowed = "move";
    },
    []
  );

  const handleDragOver = useCallback(
    (e: React.DragEvent, index: number) => {
      e.preventDefault();
      e.dataTransfer.dropEffect = "move";
      setDragOverIndex(index);
    },
    []
  );

  const handleDragEnd = useCallback(() => {
    setDragIndex(null);
    setDragOverIndex(null);
  }, []);

  const handleDrop = useCallback(
    async (e: React.DragEvent, dropIndex: number) => {
      e.preventDefault();
      if (dragIndex === null || !selectedPlaylist) return;

      const items = [...selectedPlaylist.items];
      const [moved] = items.splice(dragIndex, 1);
      items.splice(dropIndex, 0, moved);

      const newItemIds = items.map((item) => item.id);
      await reorderItems(newItemIds);
      setDragIndex(null);
      setDragOverIndex(null);
    },
    [dragIndex, selectedPlaylist, reorderItems]
  );

  // -- Helpers ----------------------------------------------------------------

  /** Find audio file name for a playlist item */
  const getFileName = useCallback(
    (audioFileId: string) => {
      const file = audioFiles.find((f) => f.id === audioFileId);
      return file?.name ?? audioFileId;
    },
    [audioFiles]
  );

  /** Find audio file duration for a playlist item */
  const getFileDuration = useCallback(
    (audioFileId: string) => {
      const file = audioFiles.find((f) => f.id === audioFileId);
      if (!file) return "--";
      const sec = file.duration_sec;
      const m = Math.floor(sec / 60);
      const s = Math.floor(sec % 60);
      return `${m}:${s.toString().padStart(2, "0")}`;
    },
    [audioFiles]
  );

  const items = selectedPlaylist?.items ?? [];
  const isPlaying = playbackStatus?.is_playing ?? false;

  return (
    <div className="flex h-full flex-col">
      {/* Playlist selector + action buttons */}
      <div className="space-y-2 px-2 pb-2">
        <div className="flex items-center gap-1">
          <div className="flex-1 min-w-0">
            <Select
              value={selectedPlaylistId ?? ""}
              onValueChange={(value) => setSelectedPlaylistId(value || null)}
            >
              <SelectTrigger className="h-8 text-sm">
                <SelectValue placeholder={t("sidebar.noPlaylists")} />
              </SelectTrigger>
              <SelectContent>
                {playlists.length === 0 ? (
                  <SelectItem value="__empty__" disabled>
                    {t("sidebar.noPlaylists")}
                  </SelectItem>
                ) : (
                  playlists.map((pl) => (
                    <SelectItem key={pl.id} value={pl.id}>
                      {pl.name} ({pl.items.length})
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>
          </div>
          <Button
            variant="outline"
            size="icon"
            className="h-8 w-8 shrink-0"
            onClick={() => setIsCreating(true)}
            title={t("playlist.create")}
          >
            <Plus className="h-3.5 w-3.5" />
          </Button>
          <Button
            variant="outline"
            size="icon"
            className="h-8 w-8 shrink-0"
            disabled={!selectedPlaylistId}
            onClick={handleDeletePlaylist}
            title={t("playlist.delete")}
          >
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
        </div>

        {/* New playlist input */}
        {isCreating && (
          <div className="flex items-center gap-1">
            <input
              ref={newPlaylistInputRef}
              type="text"
              className="h-8 flex-1 rounded-md border border-input bg-background px-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
              placeholder={t("playlist.enterName")}
              value={newPlaylistName}
              onChange={(e) => setNewPlaylistName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleCreatePlaylist();
                if (e.key === "Escape") {
                  setIsCreating(false);
                  setNewPlaylistName("");
                }
              }}
            />
            <Button
              variant="default"
              size="sm"
              className="h-8"
              disabled={!newPlaylistName.trim()}
              onClick={handleCreatePlaylist}
            >
              {t("common.save")}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              className="h-8"
              onClick={() => {
                setIsCreating(false);
                setNewPlaylistName("");
              }}
            >
              {t("common.cancel")}
            </Button>
          </div>
        )}
      </div>

      {/* Playback controls (when a playlist is selected) */}
      {selectedPlaylist && (
        <div className="space-y-1 border-y px-2 py-2">
          <div className="flex items-center gap-1">
            {isPlaying ? (
              <Button
                variant="outline"
                size="sm"
                className="h-7 gap-1"
                onClick={handleStop}
              >
                <Square className="h-3 w-3" />
                {t("playback.stop")}
              </Button>
            ) : (
              <Button
                variant="outline"
                size="sm"
                className="h-7 gap-1"
                disabled={items.length === 0}
                onClick={handlePlay}
              >
                <Play className="h-3 w-3" />
                {t("playback.play")}
              </Button>
            )}
            <Button
              variant={loopMode ? "default" : "outline"}
              size="sm"
              className="h-7 gap-1"
              onClick={() => setLoopMode(!loopMode)}
              title={t("playlist.loop")}
            >
              <Repeat className="h-3 w-3" />
              {t("playlist.loop")}
            </Button>
          </div>
          {/* Playback status */}
          {playbackStatus && playbackStatus.is_playing && (
            <div className="flex items-center gap-2">
              <Badge variant="default" className="text-xs">
                {t("playlist.playing")}
              </Badge>
              <span className="text-xs text-muted-foreground">
                {t("playlist.status", {
                  current: playbackStatus.current_item_index + 1,
                  total: playbackStatus.total_items,
                  name: playbackStatus.current_file_name,
                })}
              </span>
            </div>
          )}
        </div>
      )}

      {/* Item list */}
      <ScrollArea className="flex-1">
        {isLoading ? (
          <div className="flex items-center justify-center p-4 text-sm text-muted-foreground">
            {t("common.loading")}
          </div>
        ) : !selectedPlaylist ? (
          <div className="flex items-center justify-center p-4 text-sm text-muted-foreground">
            {t("sidebar.noPlaylists")}
          </div>
        ) : items.length === 0 ? (
          <div className="flex items-center justify-center p-4 text-sm text-muted-foreground">
            {t("playlist.empty")}
          </div>
        ) : (
          <div className="space-y-0.5 px-2 py-1">
            {items.map((item, index) => (
              <div
                key={item.id}
                draggable
                onDragStart={(e) => handleDragStart(e, index)}
                onDragOver={(e) => handleDragOver(e, index)}
                onDragEnd={handleDragEnd}
                onDrop={(e) => handleDrop(e, index)}
                className={`group flex items-start gap-1.5 rounded-md px-1.5 py-1.5 text-sm transition-colors hover:bg-accent ${
                  dragOverIndex === index ? "border-t-2 border-primary" : ""
                } ${dragIndex === index ? "opacity-50" : ""}`}
              >
                <GripVertical className="mt-0.5 h-4 w-4 shrink-0 cursor-grab text-muted-foreground" />
                <div className="flex-1 min-w-0 space-y-1">
                  <div className="flex items-center gap-1">
                    <p className="flex-1 truncate font-medium">
                      {getFileName(item.audio_file_id)}
                    </p>
                    <span className="shrink-0 text-xs text-muted-foreground">
                      {getFileDuration(item.audio_file_id)}
                    </span>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-5 w-5 shrink-0 opacity-0 group-hover:opacity-100"
                      onClick={() => handleRemoveItem(item.id)}
                      aria-label={t("common.delete")}
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  </div>
                  {/* Silence controls */}
                  <div className="flex items-center gap-2 text-xs text-muted-foreground">
                    <label className="flex items-center gap-1">
                      <span>{t("playlist.preSilence")}</span>
                      <input
                        type="number"
                        min={0}
                        max={30}
                        step={0.1}
                        className="h-5 w-14 rounded border border-input bg-background px-1 text-center text-xs"
                        value={item.pre_silence_sec}
                        onChange={(e) =>
                          handleSilenceChange(
                            item,
                            "pre",
                            parseFloat(e.target.value) || 0
                          )
                        }
                      />
                      <span>{t("common.seconds")}</span>
                    </label>
                    <label className="flex items-center gap-1">
                      <span>{t("playlist.postSilence")}</span>
                      <input
                        type="number"
                        min={0}
                        max={30}
                        step={0.1}
                        className="h-5 w-14 rounded border border-input bg-background px-1 text-center text-xs"
                        value={item.post_silence_sec}
                        onChange={(e) =>
                          handleSilenceChange(
                            item,
                            "post",
                            parseFloat(e.target.value) || 0
                          )
                        }
                      />
                      <span>{t("common.seconds")}</span>
                    </label>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </ScrollArea>

      {/* Add file button (when a playlist is selected) */}
      {selectedPlaylist && (
        <div className="border-t px-2 py-2">
          <Button
            variant="outline"
            size="sm"
            className="w-full gap-1"
            disabled={!selectedFileId}
            onClick={handleAddFile}
            title={
              selectedFileId
                ? t("playlist.addFile")
                : t("sidebar.noFiles")
            }
          >
            <FilePlus className="h-3.5 w-3.5" />
            {t("playlist.addFile")}
          </Button>
        </div>
      )}

      {/* Error display */}
      {error && (
        <div className="border-t px-2 py-1.5 text-xs text-destructive">
          {error}
        </div>
      )}
    </div>
  );
}
