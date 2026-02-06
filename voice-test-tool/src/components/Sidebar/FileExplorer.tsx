import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { FilePlus, File, X, Loader2 } from "lucide-react";
import { useAudio } from "../../hooks/useAudio";
import { Button } from "../ui/button";
import { ScrollArea } from "../ui/scroll-area";
import { Badge } from "../ui/badge";
import { cn, formatTime } from "../../lib/utils";

/** Accepted file extensions for the open-file dialog */
const AUDIO_EXTENSIONS = ["wav", "mp3", "flac", "ogg"];

export function FileExplorer() {
  const { t } = useTranslation();
  const {
    files,
    selectedFileId,
    isLoading,
    error,
    setSelectedFileId,
    loadFile,
    removeFile,
    getWaveform,
  } = useAudio();

  const [localError, setLocalError] = useState<string | null>(null);

  /** Open a native file-picker and load the selected audio file */
  const handleAddFile = useCallback(async () => {
    setLocalError(null);
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Audio",
            extensions: AUDIO_EXTENSIONS,
          },
        ],
      });

      if (selected === null) {
        // User cancelled the dialog
        return;
      }

      const filePath = typeof selected === "string" ? selected : selected;
      if (!filePath) return;

      const file = await loadFile(filePath);
      console.log("[FileExplorer] loaded file:", file.name);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setLocalError(message);
      console.log("[FileExplorer] error loading file:", message);
    }
  }, [loadFile]);

  /** Handle removing a file from the list */
  const handleRemoveFile = useCallback(
    async (fileId: string) => {
      try {
        await removeFile(fileId);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setLocalError(message);
      }
    },
    [removeFile]
  );

  /** Handle clicking a file - select it and preload waveform data */
  const handleFileClick = useCallback(
    async (fileId: string) => {
      setSelectedFileId(fileId);
      // Trigger waveform preload in background (errors are non-fatal here)
      try {
        await getWaveform(fileId);
      } catch {
        // Waveform load failure is not critical for selection
      }
    },
    [setSelectedFileId, getWaveform]
  );

  const displayError = localError || error;

  return (
    <div className="flex h-full flex-col">
      {/* Error display */}
      {displayError && (
        <div className="mx-2 mb-2 rounded-md bg-destructive/10 px-3 py-2 text-xs text-destructive">
          {displayError}
        </div>
      )}

      {/* File drop zone */}
      <div className="mx-2 mb-2 flex items-center justify-center rounded-lg border-2 border-dashed border-border p-4 text-center text-sm text-muted-foreground transition-colors hover:border-primary/50 hover:bg-accent/50">
        <p>{t("sidebar.dropFiles")}</p>
      </div>

      {/* Add file button */}
      <div className="px-2 pb-2">
        <Button
          variant="outline"
          size="sm"
          className="w-full gap-2"
          onClick={handleAddFile}
          disabled={isLoading}
        >
          {isLoading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <FilePlus className="h-4 w-4" />
          )}
          {isLoading ? t("common.loading") : t("sidebar.addFile")}
        </Button>
      </div>

      {/* File list */}
      <ScrollArea className="flex-1">
        {files.length === 0 ? (
          <div className="flex items-center justify-center p-4 text-sm text-muted-foreground">
            {t("sidebar.noFiles")}
          </div>
        ) : (
          <div className="space-y-1 px-2">
            {files.map((file) => (
              <div
                key={file.id}
                className={cn(
                  "group flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer transition-colors hover:bg-accent",
                  selectedFileId === file.id && "bg-accent text-accent-foreground"
                )}
                onClick={() => handleFileClick(file.id)}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    handleFileClick(file.id);
                  }
                }}
              >
                <File className="h-4 w-4 shrink-0 text-muted-foreground" />
                <div className="flex-1 min-w-0">
                  <p className="truncate font-medium">{file.name}</p>
                  <div className="flex items-center gap-2 text-xs text-muted-foreground">
                    <span>{formatTime(file.duration_sec)}</span>
                    <Badge variant="secondary" className="h-4 px-1 text-[10px]">
                      {file.format.toUpperCase()}
                    </Badge>
                  </div>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6 opacity-0 group-hover:opacity-100"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleRemoveFile(file.id);
                  }}
                  aria-label={t("common.delete")}
                >
                  <X className="h-3 w-3" />
                </Button>
              </div>
            ))}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
