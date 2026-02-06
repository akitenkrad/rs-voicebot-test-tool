import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { save } from "@tauri-apps/plugin-dialog";
import { ChevronDown, ChevronUp, Download } from "lucide-react";
import { useUiStore } from "../../stores/uiStore";
import { useScenario } from "../../hooks/useScenario";
import { Button } from "../ui/button";
import { Separator } from "../ui/separator";
import { LogViewer } from "./LogViewer";
import type { ReportFormat } from "../../types/scenario";

export function BottomPanel() {
  const { t } = useTranslation();
  const bottomPanelVisible = useUiStore((state) => state.bottomPanelVisible);
  const bottomPanelHeight = useUiStore((state) => state.bottomPanelHeight);
  const toggleBottomPanel = useUiStore((state) => state.toggleBottomPanel);

  const { activeSessionId, exportReport } = useScenario();
  const [isExporting, setIsExporting] = useState(false);

  const handleExport = useCallback(
    async (format: ReportFormat) => {
      if (!activeSessionId) return;

      const extensions: Record<ReportFormat, string[]> = {
        json: ["json"],
        csv: ["csv"],
        html: ["html"],
      };

      const outputPath = await save({
        filters: [
          {
            name: format.toUpperCase(),
            extensions: extensions[format],
          },
        ],
        defaultPath: `report.${format}`,
      });

      if (!outputPath) return;

      setIsExporting(true);
      try {
        await exportReport(activeSessionId, format, outputPath);
      } catch {
        // Error is set in the hook
      } finally {
        setIsExporting(false);
      }
    },
    [activeSessionId, exportReport]
  );

  return (
    <div className="flex flex-col border-t border-border bg-background">
      {/* Panel header (always visible) */}
      <div className="flex h-8 items-center justify-between px-3">
        <button
          className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground hover:text-foreground transition-colors"
          onClick={toggleBottomPanel}
        >
          {bottomPanelVisible ? (
            <ChevronDown className="h-3.5 w-3.5" />
          ) : (
            <ChevronUp className="h-3.5 w-3.5" />
          )}
          {t("log.title")}
        </button>

        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="sm"
            className="h-6 gap-1 px-2 text-xs"
            disabled={!activeSessionId || isExporting}
            onClick={() => handleExport("json")}
          >
            <Download className="h-3 w-3" />
            {t("log.export.json")}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-6 gap-1 px-2 text-xs"
            disabled={!activeSessionId || isExporting}
            onClick={() => handleExport("csv")}
          >
            <Download className="h-3 w-3" />
            {t("log.export.csv")}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-6 gap-1 px-2 text-xs"
            disabled={!activeSessionId || isExporting}
            onClick={() => handleExport("html")}
          >
            <Download className="h-3 w-3" />
            {t("log.export.html")}
          </Button>
        </div>
      </div>

      {/* Collapsible content */}
      {bottomPanelVisible && (
        <>
          <Separator />
          <div style={{ height: bottomPanelHeight }}>
            <LogViewer />
          </div>
        </>
      )}
    </div>
  );
}
