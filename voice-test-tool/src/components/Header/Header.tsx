import { useTranslation } from "react-i18next";
import { Settings, Sun, Moon, Monitor, Languages } from "lucide-react";
import { useUiStore } from "../../stores/uiStore";
import type { Theme, Language } from "../../stores/uiStore";
import { DeviceStatusIndicator } from "./DeviceStatusIndicator";
import { Button } from "../ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "../ui/tooltip";
import { Separator } from "../ui/separator";

const themeIcons: Record<Theme, React.ReactNode> = {
  light: <Sun className="h-4 w-4" />,
  dark: <Moon className="h-4 w-4" />,
  system: <Monitor className="h-4 w-4" />,
};

const themeOrder: Theme[] = ["light", "dark", "system"];

export function Header() {
  const { t, i18n } = useTranslation();
  const theme = useUiStore((state) => state.theme);
  const language = useUiStore((state) => state.language);
  const setTheme = useUiStore((state) => state.setTheme);
  const setLanguage = useUiStore((state) => state.setLanguage);
  const toggleSettings = useUiStore((state) => state.toggleSettings);

  const cycleTheme = () => {
    const currentIndex = themeOrder.indexOf(theme);
    const nextIndex = (currentIndex + 1) % themeOrder.length;
    setTheme(themeOrder[nextIndex]);
  };

  const toggleLanguage = () => {
    const newLang: Language = language === "ja" ? "en" : "ja";
    setLanguage(newLang);
    i18n.changeLanguage(newLang);
  };

  const themeLabel = t(`settings.theme${theme.charAt(0).toUpperCase() + theme.slice(1)}`);

  return (
    <header className="flex h-14 items-center justify-between border-b border-border bg-background px-4">
      {/* Left: Logo and title */}
      <div className="flex items-center gap-3">
        <h1 className="text-lg font-semibold text-foreground">
          {t("app.title")}
        </h1>
      </div>

      {/* Center: Device status */}
      <div className="flex items-center">
        <DeviceStatusIndicator />
      </div>

      {/* Right: Controls */}
      <div className="flex items-center gap-1">
        {/* Language toggle */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={toggleLanguage}
              aria-label={t("settings.language")}
            >
              <Languages className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <p>{t("settings.language")}: {language.toUpperCase()}</p>
          </TooltipContent>
        </Tooltip>

        {/* Theme toggle */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={cycleTheme}
              aria-label={themeLabel}
            >
              {themeIcons[theme]}
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <p>{t("settings.theme")}: {themeLabel}</p>
          </TooltipContent>
        </Tooltip>

        <Separator orientation="vertical" className="mx-1 h-6" />

        {/* Settings button */}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={toggleSettings}
              aria-label={t("header.settings")}
            >
              <Settings className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <p>{t("header.settings")}</p>
          </TooltipContent>
        </Tooltip>
      </div>
    </header>
  );
}
