import { useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Play,
  Trash2,
  Upload,
  Pause,
  RotateCcw,
  XCircle,
} from "lucide-react";
import { Button } from "../ui/button";
import { ScrollArea } from "../ui/scroll-area";
import { Badge } from "../ui/badge";
import { Progress } from "../ui/progress";
import { cn } from "../../lib/utils";
import { useScenario } from "../../hooks/useScenario";
import type { ScenarioProgressInfo } from "../../types/scenario";

const statusColorMap: Record<ScenarioProgressInfo["status"], string> = {
  running: "bg-device-active",
  paused: "bg-yellow-500",
  completed: "bg-device-active",
  failed: "bg-device-error",
  aborted: "bg-device-inactive",
};

const statusBadgeVariant: Record<
  ScenarioProgressInfo["status"],
  "default" | "secondary" | "destructive"
> = {
  running: "default",
  paused: "secondary",
  completed: "default",
  failed: "destructive",
  aborted: "secondary",
};

export function ScenarioPanel() {
  const { t } = useTranslation();
  const {
    scenarios,
    selectedScenarioId,
    selectedScenario,
    executionStatus,
    isLoading,
    error,
    fetchScenarios,
    loadScenarioFile,
    deleteScenario,
    executeScenario,
    pauseExecution,
    resumeExecution,
    abortExecution,
    setSelectedScenarioId,
  } = useScenario();

  // Load scenarios on mount
  useEffect(() => {
    fetchScenarios();
  }, [fetchScenarios]);

  // Whether a scenario is currently being executed
  const isExecutionActive =
    executionStatus?.status === "running" ||
    executionStatus?.status === "paused";

  // -- Handlers --

  const handleLoadScenario = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Scenario",
          extensions: ["json", "yaml", "yml"],
        },
      ],
    });
    if (selected) {
      try {
        const info = await loadScenarioFile(selected as string);
        setSelectedScenarioId(info.id);
      } catch {
        // Error is set in the hook
      }
    }
  }, [loadScenarioFile, setSelectedScenarioId]);

  const handleExecute = useCallback(
    async (scenarioId: string) => {
      try {
        await executeScenario(scenarioId);
      } catch {
        // Error is set in the hook
      }
    },
    [executeScenario]
  );

  const handleDelete = useCallback(
    async (scenarioId: string) => {
      if (!window.confirm(t("scenario.confirmDelete"))) return;
      try {
        await deleteScenario(scenarioId);
      } catch {
        // Error is set in the hook
      }
    },
    [deleteScenario, t]
  );

  const handlePause = useCallback(async () => {
    try {
      await pauseExecution();
    } catch {
      // Error is set in the hook
    }
  }, [pauseExecution]);

  const handleResume = useCallback(async () => {
    try {
      await resumeExecution();
    } catch {
      // Error is set in the hook
    }
  }, [resumeExecution]);

  const handleAbort = useCallback(async () => {
    try {
      await abortExecution();
    } catch {
      // Error is set in the hook
    }
  }, [abortExecution]);

  // Progress percentage for the progress bar
  const progressPercent =
    executionStatus && executionStatus.total_turns > 0
      ? Math.round(
          (executionStatus.current_turn / executionStatus.total_turns) * 100
        )
      : 0;

  return (
    <div className="flex h-full flex-col">
      {/* Load scenario button */}
      <div className="px-2 pb-2">
        <Button
          variant="outline"
          size="sm"
          className="w-full gap-2"
          onClick={handleLoadScenario}
          disabled={isLoading}
        >
          <Upload className="h-4 w-4" />
          {t("scenario.load")}
        </Button>
      </div>

      {/* Execution controls (when a scenario has an active execution) */}
      {executionStatus && (
        <div className="space-y-2 border-y px-2 py-2">
          <div className="flex items-center gap-1">
            {executionStatus.status === "running" ? (
              <Button
                variant="outline"
                size="sm"
                className="h-7 gap-1"
                onClick={handlePause}
              >
                <Pause className="h-3 w-3" />
                {t("scenario.pause")}
              </Button>
            ) : executionStatus.status === "paused" ? (
              <Button
                variant="outline"
                size="sm"
                className="h-7 gap-1"
                onClick={handleResume}
              >
                <RotateCcw className="h-3 w-3" />
                {t("scenario.resume")}
              </Button>
            ) : null}
            {isExecutionActive && (
              <Button
                variant="outline"
                size="sm"
                className="h-7 gap-1 text-destructive hover:text-destructive"
                onClick={handleAbort}
              >
                <XCircle className="h-3 w-3" />
                {t("scenario.abort")}
              </Button>
            )}
            <Badge
              variant={statusBadgeVariant[executionStatus.status]}
              className="ml-auto h-5 text-[10px]"
            >
              {t(`scenario.status.${executionStatus.status}`)}
            </Badge>
          </div>
          {/* Progress display */}
          <div className="space-y-1">
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span>
                {t("scenario.progress", {
                  current: executionStatus.current_turn,
                  total: executionStatus.total_turns,
                })}
              </span>
              {executionStatus.current_file_name && (
                <span className="truncate ml-2 max-w-[120px]">
                  {executionStatus.current_file_name}
                </span>
              )}
            </div>
            <Progress value={progressPercent} className="h-1.5" />
          </div>
        </div>
      )}

      {/* Scenario list */}
      <ScrollArea className="flex-1">
        {isLoading && scenarios.length === 0 ? (
          <div className="flex items-center justify-center p-4 text-sm text-muted-foreground">
            {t("common.loading")}
          </div>
        ) : scenarios.length === 0 ? (
          <div className="flex items-center justify-center p-4 text-sm text-muted-foreground">
            {t("sidebar.noScenarios")}
          </div>
        ) : (
          <div className="space-y-1 px-2">
            {scenarios.map((scenario) => {
              const isSelected = selectedScenarioId === scenario.id;
              const isRunningThis =
                executionStatus?.scenario_id === scenario.id &&
                isExecutionActive;

              return (
                <div
                  key={scenario.id}
                  className={cn(
                    "group flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer transition-colors hover:bg-accent",
                    isSelected && "bg-accent text-accent-foreground"
                  )}
                  onClick={() => setSelectedScenarioId(scenario.id)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      setSelectedScenarioId(scenario.id);
                    }
                  }}
                >
                  {/* Status dot */}
                  {executionStatus?.scenario_id === scenario.id && (
                    <span
                      className={cn(
                        "h-2 w-2 shrink-0 rounded-full",
                        statusColorMap[executionStatus.status]
                      )}
                    />
                  )}

                  <div className="flex-1 min-w-0">
                    <p className="truncate font-medium">{scenario.name}</p>
                    <p className="text-xs text-muted-foreground">
                      {scenario.turn_count} {t("scenario.turn")}
                    </p>
                  </div>

                  {/* Action buttons (visible on hover, hidden during execution) */}
                  <div className="flex gap-0.5 opacity-0 group-hover:opacity-100">
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6"
                      aria-label={t("scenario.execute")}
                      disabled={isExecutionActive}
                      onClick={(e) => {
                        e.stopPropagation();
                        handleExecute(scenario.id);
                      }}
                    >
                      <Play className="h-3 w-3" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6"
                      aria-label={t("scenario.delete")}
                      disabled={isRunningThis}
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDelete(scenario.id);
                      }}
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  </div>

                  {/* Running badge */}
                  {isRunningThis && (
                    <Badge variant="default" className="h-5 text-[10px]">
                      {t("scenario.status.running")}
                    </Badge>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </ScrollArea>

      {/* Selected scenario details (when selected, not executing) */}
      {selectedScenario && !isExecutionActive && (
        <div className="border-t px-2 py-2 space-y-2">
          <div>
            <p className="text-sm font-medium">{selectedScenario.name}</p>
            {selectedScenario.description && (
              <p className="text-xs text-muted-foreground mt-0.5">
                {selectedScenario.description}
              </p>
            )}
          </div>
          <div className="text-xs text-muted-foreground">
            {selectedScenario.turn_count} {t("scenario.turn")}
          </div>
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
