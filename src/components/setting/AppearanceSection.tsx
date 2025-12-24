import { useState, useEffect } from "react";
import { useSetting, ThemeMode } from "@/contexts/SettingContext";
import { Sun, Moon, Monitor, Check } from "lucide-react";
import { cn } from "@/lib/utils";

export default function AppearanceSection() {
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
      console.error("更改主题失败:", error);
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
    <div className="space-y-6">
      <div className="space-y-4">
        <h4 className="text-base font-medium px-2">外观设置</h4>
        <div className="grid grid-cols-3 gap-4 px-2">
          <ThemeOption value="light" icon={Sun} label="浅色" />
          <ThemeOption value="dark" icon={Moon} label="深色" />
          <ThemeOption value="system" icon={Monitor} label="跟随系统" />
        </div>
      </div>

      <div className="space-y-4">
        <h4 className="text-base font-medium px-2">主题色</h4>
        <div className="grid grid-cols-5 gap-4 px-2">
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
      </div>
    </div>
  );
}
