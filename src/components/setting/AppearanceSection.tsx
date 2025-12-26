import { useState, useEffect } from "react";
import { useSetting, ThemeMode } from "@/contexts/SettingContext";
import { Sun, Moon, Monitor, Check } from "lucide-react";
import { cn } from "@/lib/utils";
import { Card, CardContent } from "@/components/ui/card";
import { useTranslation } from "react-i18next";

/**
 * Renders the appearance settings section with controls for selecting theme mode and theme color.
 *
 * @returns A React element containing theme mode options (light, dark, system) and theme color swatches, wired to update the user's general settings.
 */
export default function AppearanceSection() {
  const { t } = useTranslation();
  const { setting, updateGeneralSetting } = useSetting();
  const [theme, setTheme] = useState<ThemeMode>("system");

  useEffect(() => {
    if (setting?.general) {
      setTheme(setting.general.theme || "system");
    }
  }, [setting]);

  const handleThemeChange = async (newTheme: ThemeMode) => {
    try {
      await updateGeneralSetting({ theme: newTheme });
      setTheme(newTheme);
    } catch (error) {
      console.error("Failed to change theme:", error);
    }
  };

  const ThemeOption = ({
    value,
    icon: Icon,
    label,
  }: {
    value: ThemeMode;
    icon: any;
    label: string;
  }) => (
    <div
      onClick={() => handleThemeChange(value)}
      className={cn(
        "cursor-pointer relative flex flex-col items-center gap-2 p-4 rounded-xl border-2 transition-all",
        theme === value
          ? "border-primary bg-primary/5"
          : "border-transparent bg-muted/50 hover:bg-muted"
      )}
    >
      <div
        className={cn(
          "p-2 rounded-full",
          theme === value
            ? "bg-primary/10 text-primary"
            : "bg-transparent text-muted-foreground"
        )}
      >
        <Icon className="w-6 h-6" />
      </div>
      <span
        className={cn(
          "text-sm font-medium",
          theme === value ? "text-primary" : "text-muted-foreground"
        )}
      >
        {label}
      </span>
      {theme === value && (
        <div className="absolute top-2 right-2 text-primary">
          <Check className="w-4 h-4" />
        </div>
      )}
    </div>
  );

  return (
    <>
      <Card>
        <div className="flex items-center gap-4 mb-4 px-6 pt-6">
          <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">
            {t("settings.sections.appearance.themeMode.title")}
          </h3>
          <div className="h-px flex-1 bg-border/50"></div>
        </div>
        <CardContent className="pt-0">
          <div className="grid grid-cols-3 gap-4">
            <ThemeOption value="light" icon={Sun} label={t("settings.sections.appearance.themeMode.light")} />
            <ThemeOption value="dark" icon={Moon} label={t("settings.sections.appearance.themeMode.dark")} />
            <ThemeOption value="system" icon={Monitor} label={t("settings.sections.appearance.themeMode.system")} />
          </div>
        </CardContent>
      </Card>

      <Card>
        <div className="flex items-center gap-4 mb-4 px-6 pt-6">
          <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">
            {t("settings.sections.appearance.themeColor.title")}
          </h3>
          <div className="h-px flex-1 bg-border/50"></div>
        </div>
        <CardContent className="pt-0">
          <div className="grid grid-cols-5 gap-4">
            {[
              { name: "catppuccin", color: "#cba6f7" },
              { name: "zinc", color: "#52525b" },
              { name: "t3chat", color: "#a3004c" },
            ].map((item) => (
              <div
                key={item.name}
                onClick={() => {
                  updateGeneralSetting({ theme_color: item.name });
                }}
                className={cn(
                  "cursor-pointer group relative flex flex-col items-center gap-2 p-2 rounded-xl border-2 transition-all hover:bg-muted/50",
                  setting?.general?.theme_color === item.name ||
                    (item.name === "catppuccin" && !setting?.general?.theme_color)
                    ? "border-primary bg-primary/5"
                    : "border-transparent"
                )}
              >
                <div
                  className="w-8 h-8 rounded-full shadow-sm"
                  style={{ backgroundColor: item.color }}
                />
                <span className="text-xs font-medium capitalize text-muted-foreground group-hover:text-foreground">
                  {item.name}
                </span>
                {(setting?.general?.theme_color === item.name ||
                  (item.name === "catppuccin" &&
                    !setting?.general?.theme_color)) && (
                  <div className="absolute top-1 right-1 text-primary bg-background rounded-full p-0.5 shadow-sm">
                    <Check className="w-3 h-3" />
                  </div>
                )}
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </>
  );
}