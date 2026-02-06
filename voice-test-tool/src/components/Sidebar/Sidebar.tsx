import { useTranslation } from "react-i18next";
import { FolderOpen, ListMusic, FileText, MessageSquare } from "lucide-react";
import { useUiStore } from "../../stores/uiStore";
import type { SidebarTab } from "../../stores/uiStore";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { FileExplorer } from "./FileExplorer";
import { PlaylistPanel } from "./PlaylistPanel";
import { ScenarioPanel } from "./ScenarioPanel";
import { TtsTestPanel } from "./TtsTestPanel";
import { cn } from "../../lib/utils";

const tabIcons: Record<SidebarTab, React.ReactNode> = {
  files: <FolderOpen className="h-4 w-4" />,
  playlists: <ListMusic className="h-4 w-4" />,
  scenarios: <FileText className="h-4 w-4" />,
  ttsTest: <MessageSquare className="h-4 w-4" />,
};

export function Sidebar() {
  const { t } = useTranslation();
  const sidebarVisible = useUiStore((state) => state.sidebarVisible);
  const activeSidebarTab = useUiStore((state) => state.activeSidebarTab);
  const setActiveSidebarTab = useUiStore((state) => state.setActiveSidebarTab);

  if (!sidebarVisible) {
    return null;
  }

  return (
    <aside
      className={cn(
        "flex h-full w-[250px] shrink-0 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground"
      )}
    >
      <Tabs
        value={activeSidebarTab}
        onValueChange={(value) => setActiveSidebarTab(value as SidebarTab)}
        className="flex h-full flex-col"
      >
        <TabsList className="mx-2 mt-2 grid w-auto grid-cols-4">
          <TabsTrigger value="files" className="gap-1.5 text-xs">
            {tabIcons.files}
            {t("sidebar.files")}
          </TabsTrigger>
          <TabsTrigger value="playlists" className="gap-1.5 text-xs">
            {tabIcons.playlists}
            {t("sidebar.playlists")}
          </TabsTrigger>
          <TabsTrigger value="scenarios" className="gap-1.5 text-xs">
            {tabIcons.scenarios}
            {t("sidebar.scenarios")}
          </TabsTrigger>
          <TabsTrigger value="ttsTest" className="gap-1.5 text-xs">
            {tabIcons.ttsTest}
            {t("sidebar.ttsTest")}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="files" className="flex-1 overflow-hidden mt-0 pt-2">
          <FileExplorer />
        </TabsContent>

        <TabsContent value="playlists" className="flex-1 overflow-hidden mt-0 pt-2">
          <PlaylistPanel />
        </TabsContent>

        <TabsContent value="scenarios" className="flex-1 overflow-hidden mt-0 pt-2">
          <ScenarioPanel />
        </TabsContent>

        <TabsContent value="ttsTest" className="flex-1 overflow-hidden mt-0 pt-2">
          <TtsTestPanel />
        </TabsContent>
      </Tabs>
    </aside>
  );
}
