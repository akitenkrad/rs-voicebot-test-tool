import { useEffect, useRef, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { useScenario } from "../../hooks/useScenario";
import { ScrollArea } from "../ui/scroll-area";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import { cn } from "../../lib/utils";
import type { LogLevel } from "../../types/scenario";

const levelColors: Record<LogLevel, string> = {
  info: "bg-blue-500/10 text-blue-600 dark:text-blue-400",
  warn: "bg-yellow-500/10 text-yellow-600 dark:text-yellow-400",
  error: "bg-red-500/10 text-red-600 dark:text-red-400",
};

const levelBadgeVariants: Record<LogLevel, "default" | "secondary" | "destructive"> = {
  info: "secondary",
  warn: "default",
  error: "destructive",
};

export function LogViewer() {
  const { t } = useTranslation();
  const {
    logs,
    activeSessionId,
    testSessions,
    fetchLogs,
    fetchTestSessions,
    clearLogs,
  } = useScenario();

  const scrollRef = useRef<HTMLDivElement>(null);
  const [filterLevel, setFilterLevel] = useState<LogLevel | "all">("all");
  const [viewingSessionId, setViewingSessionId] = useState<string | null>(null);

  // Load test sessions on mount
  useEffect(() => {
    fetchTestSessions();
  }, [fetchTestSessions]);

  // When active session changes, switch to viewing it
  useEffect(() => {
    if (activeSessionId) {
      setViewingSessionId(activeSessionId);
    }
  }, [activeSessionId]);

  // When viewing session changes, fetch its logs
  const handleSessionChange = useCallback(
    (sessionId: string) => {
      setViewingSessionId(sessionId);
      if (sessionId) {
        fetchLogs(sessionId);
      } else {
        clearLogs();
      }
    },
    [fetchLogs, clearLogs]
  );

  const filteredLogs =
    filterLevel === "all"
      ? logs
      : logs.filter((log) => log.level === filterLevel);

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs]);

  const levels: Array<LogLevel | "all"> = ["all", "info", "warn", "error"];

  return (
    <div className="flex h-full flex-col">
      {/* Filter bar + session selector */}
      <div className="flex items-center gap-2 px-2 pb-2">
        {/* Session selector */}
        <div className="min-w-0 w-48 shrink-0">
          <Select
            value={viewingSessionId ?? ""}
            onValueChange={(value) => handleSessionChange(value)}
          >
            <SelectTrigger className="h-6 text-xs">
              <SelectValue placeholder={t("log.selectSession")} />
            </SelectTrigger>
            <SelectContent>
              {testSessions.length === 0 ? (
                <SelectItem value="__empty__" disabled>
                  {t("log.noSessions")}
                </SelectItem>
              ) : (
                testSessions.map((session) => (
                  <SelectItem key={session.id} value={session.id}>
                    {session.scenario_name} ({session.status})
                  </SelectItem>
                ))
              )}
            </SelectContent>
          </Select>
        </div>

        {/* Level filters */}
        {levels.map((level) => (
          <Button
            key={level}
            variant={filterLevel === level ? "secondary" : "ghost"}
            size="sm"
            className="h-6 px-2 text-xs"
            onClick={() => setFilterLevel(level)}
          >
            {level === "all"
              ? "All"
              : t(`log.level.${level}`)}
          </Button>
        ))}
        <div className="flex-1" />
        <Button
          variant="ghost"
          size="sm"
          className="h-6 px-2 text-xs"
          onClick={clearLogs}
        >
          {t("log.clear")}
        </Button>
      </div>

      {/* Log entries */}
      <ScrollArea className="flex-1" ref={scrollRef}>
        {filteredLogs.length === 0 ? (
          <div className="flex items-center justify-center p-4 text-sm text-muted-foreground">
            {t("log.noLogs")}
          </div>
        ) : (
          <div className="space-y-0.5 px-2 font-mono text-xs">
            {filteredLogs.map((log) => (
              <div
                key={log.id}
                className={cn(
                  "flex items-start gap-2 rounded px-2 py-1",
                  levelColors[log.level]
                )}
              >
                <span className="shrink-0 tabular-nums text-muted-foreground">
                  {new Date(log.timestamp).toLocaleTimeString()}
                </span>
                <Badge
                  variant={levelBadgeVariants[log.level]}
                  className="h-4 shrink-0 px-1 text-[10px] font-normal"
                >
                  {log.level.toUpperCase()}
                </Badge>
                <span className="flex-1 break-all">{log.message}</span>
              </div>
            ))}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
