import { useCallback, useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import {
  FileUp,
  Play,
  Square,
  Loader2,
  FileText,
  Pause,
  Download,
  CheckCircle,
  XCircle,
  Clock,
} from "lucide-react";
import { useTtsTest } from "../../hooks/useTtsTest";
import { Button } from "../ui/button";
import { ScrollArea } from "../ui/scroll-area";
import { Badge } from "../ui/badge";
import { Progress } from "../ui/progress";
import { cn } from "../../lib/utils";
import type { TtsTestStatus } from "../../lib/tauri";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

/** Map TTS test status type to color for the status badge */
const statusBadgeVariant: Record<
  TtsTestStatus["type"],
  "default" | "secondary" | "destructive"
> = {
  Pending: "secondary",
  Running: "default",
  Paused: "secondary",
  Completed: "default",
  Aborted: "secondary",
  Failed: "destructive",
};

export function TtsTestPanel() {
  const { t } = useTranslation();
  const {
    testCases,
    selectedCsvPath,
    selectedTestCaseId,
    isLoading,
    error,
    previewingId,
    currentSession,
    setSelectedTestCaseId,
    loadCsv,
    loadCsvFromPath,
    previewTts,
    stopPreview,
    startTest,
    pauseTest,
    resumeTest,
    abortTest,
    exportResults,
  } = useTtsTest();

  const [localError, setLocalError] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);

  // Listen for drag-drop events from Tauri
  useEffect(() => {
    const webview = getCurrentWebviewWindow();
    const unlisten = webview.onDragDropEvent(async (event) => {
      if (event.payload.type === "enter" || event.payload.type === "over") {
        setIsDragging(true);
      } else if (event.payload.type === "drop") {
        setIsDragging(false);
        const paths = event.payload.paths;
        if (paths && paths.length > 0) {
          const csvPath = paths.find((p) => p.toLowerCase().endsWith(".csv"));
          if (csvPath) {
            try {
              await loadCsvFromPath(csvPath);
            } catch (err) {
              setLocalError(err instanceof Error ? err.message : String(err));
            }
          } else {
            setLocalError(t("tts.error.notCsv"));
          }
        }
      } else {
        // Handle leave and any other events
        setIsDragging(false);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [loadCsvFromPath, t]);

  // Determine current status
  const status = currentSession?.status;
  const isRunning = status?.type === "Running";
  const isPaused = status?.type === "Paused";
  const isCompleted = status?.type === "Completed";
  const isAborted = status?.type === "Aborted";
  const isFailed = status?.type === "Failed";
  const isExecutionActive = isRunning || isPaused;

  // Current index for progress display
  const currentIndex =
    status?.type === "Running" || status?.type === "Paused"
      ? status.current_index
      : -1;

  // Progress percentage
  const progressPercent =
    testCases.length > 0 && currentIndex >= 0
      ? Math.round(((currentIndex + 1) / testCases.length) * 100)
      : isCompleted
        ? 100
        : 0;

  // Get result for a test case
  const getResultForCase = (caseId: string) => {
    return currentSession?.results.find((r) => r.test_case_id === caseId);
  };

  /** Handle loading a CSV file */
  const handleLoadCsv = useCallback(async () => {
    setLocalError(null);
    try {
      await loadCsv();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setLocalError(message);
    }
  }, [loadCsv]);

  /** Handle preview button click */
  const handlePreview = useCallback(
    async (testCaseId: string) => {
      const testCase = testCases.find((tc) => tc.id === testCaseId);
      if (!testCase) return;

      setLocalError(null);
      try {
        await previewTts(testCase);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setLocalError(message);
      }
    },
    [testCases, previewTts]
  );

  /** Handle start test */
  const handleStartTest = useCallback(async () => {
    setLocalError(null);
    try {
      await startTest();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setLocalError(message);
    }
  }, [startTest]);

  /** Handle pause test */
  const handlePauseTest = useCallback(async () => {
    try {
      await pauseTest();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setLocalError(message);
    }
  }, [pauseTest]);

  /** Handle resume test */
  const handleResumeTest = useCallback(async () => {
    try {
      await resumeTest();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setLocalError(message);
    }
  }, [resumeTest]);

  /** Handle abort test */
  const handleAbortTest = useCallback(async () => {
    try {
      await abortTest();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setLocalError(message);
    }
  }, [abortTest]);

  /** Handle export results */
  const handleExportResults = useCallback(async () => {
    setLocalError(null);
    try {
      const result = await exportResults();
      if (result) {
        console.log(`[TtsTestPanel] Exported to: ${result}`);
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setLocalError(message);
    }
  }, [exportResults]);

  /** Truncate text for display */
  const truncateText = (text: string, maxLength: number = 50) => {
    if (text.length <= maxLength) return text;
    return text.slice(0, maxLength) + "...";
  };

  const displayError = localError || error;

  return (
    <div className={cn("flex h-full flex-col relative", isDragging && "ring-2 ring-primary ring-inset")}>
      {/* Drop zone overlay */}
      {isDragging && (
        <div className="absolute inset-0 z-10 flex items-center justify-center bg-primary/10 backdrop-blur-sm">
          <div className="flex flex-col items-center gap-2 text-primary">
            <FileUp className="h-8 w-8" />
            <span className="text-sm font-medium">{t("tts.dropCsv")}</span>
          </div>
        </div>
      )}

      {/* Error display */}
      {displayError && (
        <div className="mx-2 mb-2 rounded-md bg-destructive/10 px-3 py-2 text-xs text-destructive">
          {displayError}
        </div>
      )}

      {/* CSV file info */}
      {selectedCsvPath && (
        <div className="mx-2 mb-2 rounded-md bg-muted px-3 py-2">
          <p className="text-xs text-muted-foreground truncate">
            {selectedCsvPath.split("/").pop()}
          </p>
          <Badge variant="secondary" className="mt-1 h-4 px-1 text-[10px]">
            {testCases.length} {t("scenario.turn")}
          </Badge>
        </div>
      )}

      {/* Load CSV button */}
      <div className="px-2 pb-2">
        <Button
          variant="outline"
          size="sm"
          className="w-full gap-2"
          onClick={handleLoadCsv}
          disabled={isLoading || isExecutionActive}
        >
          {isLoading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <FileUp className="h-4 w-4" />
          )}
          {isLoading ? t("common.loading") : t("sidebar.loadCsv")}
        </Button>
      </div>

      {/* Execution controls */}
      {(testCases.length > 0 || currentSession) && (
        <div className="space-y-2 border-y px-2 py-2">
          <div className="flex items-center gap-1">
            {/* Start button - show when not executing */}
            {!isExecutionActive && !isCompleted && !isAborted && !isFailed && (
              <Button
                variant="outline"
                size="sm"
                className="h-7 gap-1"
                onClick={handleStartTest}
                disabled={testCases.length === 0 || isLoading}
              >
                <Play className="h-3 w-3" />
                {t("tts.actions.start")}
              </Button>
            )}

            {/* Pause button - show when running */}
            {isRunning && (
              <Button
                variant="outline"
                size="sm"
                className="h-7 gap-1"
                onClick={handlePauseTest}
              >
                <Pause className="h-3 w-3" />
                {t("tts.actions.pause")}
              </Button>
            )}

            {/* Resume button - show when paused */}
            {isPaused && (
              <Button
                variant="outline"
                size="sm"
                className="h-7 gap-1"
                onClick={handleResumeTest}
              >
                <Play className="h-3 w-3" />
                {t("tts.actions.resume")}
              </Button>
            )}

            {/* Abort button - show when executing */}
            {isExecutionActive && (
              <Button
                variant="outline"
                size="sm"
                className="h-7 gap-1 text-destructive hover:text-destructive"
                onClick={handleAbortTest}
              >
                <Square className="h-3 w-3" />
                {t("tts.actions.abort")}
              </Button>
            )}

            {/* Export button - show when completed/aborted/failed */}
            {(isCompleted || isAborted || isFailed) && currentSession && (
              <Button
                variant="outline"
                size="sm"
                className="h-7 gap-1"
                onClick={handleExportResults}
                disabled={isLoading}
              >
                <Download className="h-3 w-3" />
                {t("tts.actions.export")}
              </Button>
            )}

            {/* Status badge */}
            {status && (
              <Badge
                variant={statusBadgeVariant[status.type]}
                className="ml-auto h-5 text-[10px]"
              >
                {t(`tts.status.${status.type.toLowerCase()}`)}
              </Badge>
            )}
          </div>

          {/* Progress display */}
          {(isExecutionActive || isCompleted) && (
            <div className="space-y-1">
              <div className="flex items-center justify-between text-xs text-muted-foreground">
                <span>
                  {t("tts.progress", {
                    current: isCompleted
                      ? testCases.length
                      : Math.min(currentIndex + 1, testCases.length),
                    total: testCases.length,
                  })}
                </span>
              </div>
              <Progress value={progressPercent} className="h-1.5" />
            </div>
          )}
        </div>
      )}

      {/* Test cases list */}
      <ScrollArea className="flex-1">
        {testCases.length === 0 ? (
          <div className="flex items-center justify-center p-4 text-sm text-muted-foreground">
            {t("sidebar.noTestCases")}
          </div>
        ) : (
          <div className="space-y-1 px-2">
            {testCases.map((testCase, index) => {
              const isPreviewing = previewingId === testCase.id;
              const isSelected = selectedTestCaseId === testCase.id;
              const result = getResultForCase(testCase.id);
              const isCurrent = index === currentIndex;
              const hasError = result?.error;
              const isComplete = result && !hasError;

              return (
                <div
                  key={testCase.id}
                  className={cn(
                    "group flex items-start gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer transition-colors hover:bg-accent",
                    isSelected && "bg-accent text-accent-foreground",
                    isCurrent && isRunning && "border border-primary",
                    isCurrent && isPaused && "border border-yellow-500",
                    hasError && "border border-destructive/50",
                    isComplete && "border border-green-500/50"
                  )}
                  onClick={() => setSelectedTestCaseId(testCase.id)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      setSelectedTestCaseId(testCase.id);
                    }
                  }}
                >
                  {/* Status icon */}
                  <div className="mt-0.5 shrink-0">
                    {isCurrent && isRunning && (
                      <Loader2 className="h-4 w-4 animate-spin text-primary" />
                    )}
                    {isCurrent && isPaused && (
                      <Pause className="h-4 w-4 text-yellow-500" />
                    )}
                    {!isCurrent && isComplete && (
                      <CheckCircle className="h-4 w-4 text-green-500" />
                    )}
                    {!isCurrent && hasError && (
                      <XCircle className="h-4 w-4 text-destructive" />
                    )}
                    {!isCurrent && !result && (
                      <FileText className="h-4 w-4 text-muted-foreground" />
                    )}
                  </div>

                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <Badge
                        variant="outline"
                        className="h-4 px-1 text-[10px] font-mono"
                      >
                        {testCase.id}
                      </Badge>
                    </div>
                    <p className="mt-1 text-xs text-muted-foreground break-words">
                      {truncateText(testCase.text, 80)}
                    </p>
                    {/* Show timing info if available */}
                    {result && (
                      <p className="mt-0.5 text-[10px] text-muted-foreground">
                        TTS: {result.tts_duration_sec.toFixed(1)}s
                        {result.response_duration_sec !== null &&
                          ` | Resp: ${result.response_duration_sec.toFixed(1)}s`}
                      </p>
                    )}
                    {/* Show error if any */}
                    {hasError && (
                      <p className="mt-0.5 text-[10px] text-destructive truncate">
                        {result.error}
                      </p>
                    )}
                  </div>

                  {/* Preview button - only show when not executing */}
                  {!isExecutionActive && (
                    <Button
                      variant="ghost"
                      size="icon"
                      className={cn(
                        "h-6 w-6 shrink-0",
                        !isPreviewing && "opacity-0 group-hover:opacity-100"
                      )}
                      onClick={(e) => {
                        e.stopPropagation();
                        if (isPreviewing) {
                          stopPreview();
                        } else {
                          handlePreview(testCase.id);
                        }
                      }}
                      aria-label={
                        isPreviewing
                          ? t("sidebar.stopPreview")
                          : t("sidebar.preview")
                      }
                    >
                      {isPreviewing ? (
                        <Square className="h-3 w-3" />
                      ) : (
                        <Play className="h-3 w-3" />
                      )}
                    </Button>
                  )}

                  {/* Current indicator when running */}
                  {isCurrent && isExecutionActive && (
                    <Clock className="h-4 w-4 shrink-0 text-primary" />
                  )}
                </div>
              );
            })}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
