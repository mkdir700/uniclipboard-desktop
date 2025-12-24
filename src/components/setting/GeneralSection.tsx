import { useState, useEffect } from "react";
import { Switch } from "@/components/ui";
import { useSetting, ThemeMode } from "@/contexts/SettingContext";
import { invoke } from "@tauri-apps/api/core";
import { Sun, Moon, Monitor, Check } from "lucide-react";
import { cn } from "@/lib/utils";

export default function GeneralSection() {
  const { setting, updateGeneralSetting } = useSetting();
  const [autoStart, setAutoStart] = useState(false);
  const [silentStart, setSilentStart] = useState(false);
  const [theme, setTheme] = useState<ThemeMode>("system");
  const [isLoading, setIsLoading] = useState(true);

  // 初始化时检查自启动状态和设置
  useEffect(() => {
    const checkStatus = async () => {
      try {
        setIsLoading(true);
        // 检查系统自启动状态
        const enabled = await invoke("is_autostart_enabled");
        setAutoStart(enabled as boolean);
        
        // 从配置中读取静默启动状态和主题
        if (setting?.general) {
          setSilentStart(setting.general.silent_start);
          setTheme(setting.general.theme || "system");
        }
      } catch (error) {
        console.error("初始化设置失败:", error);
      } finally {
        setIsLoading(false);
      }
    };

    checkStatus();
  }, [setting]);

  // 处理自启动开关变化
  const handleAutoStartChange = async (checked: boolean) => {
    try {
      setIsLoading(true);
      const newState = checked;

      if (newState) {
        await invoke("enable_autostart");
      } else {
        await invoke("disable_autostart");
      }

      // 更新设置和状态
      await updateGeneralSetting({ auto_start: newState });
      setAutoStart(newState);
    } catch (error) {
      console.error("更改自启动状态失败:", error);
    } finally {
      setIsLoading(false);
    }
  };

  // 处理静默启动开关变化
  const handleSilentStartChange = async (checked: boolean) => {
    try {
      // 更新设置和状态
      await updateGeneralSetting({ silent_start: checked });
      setSilentStart(checked);
    } catch (error) {
      console.error("更改静默启动状态失败:", error);
    }
  };

  // 处理主题变化
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
      <div className={cn(
        "p-2 rounded-full",
        theme === value ? "bg-primary/10 text-primary" : "bg-transparent text-muted-foreground"
      )}>
        <Icon className="w-6 h-6" />
      </div>
      <span className={cn(
        "text-sm font-medium",
        theme === value ? "text-primary" : "text-muted-foreground"
      )}>
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
        <h4 className="text-base font-medium px-2">启动设置</h4>
        
        <div className="py-2 rounded-lg px-2">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <h4 className="text-sm font-medium">开机自启动</h4>
              <p className="text-xs text-muted-foreground">
                系统启动时自动启动Uniclipboard
              </p>
            </div>
            <Switch
              checked={autoStart}
              onCheckedChange={handleAutoStartChange}
              disabled={isLoading}
            />
          </div>
        </div>

        <div className="py-2 rounded-lg px-2">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <h4 className="text-sm font-medium">静默启动</h4>
              <p className="text-xs text-muted-foreground">
                启动时不显示主界面，在后台运行
              </p>
            </div>
            <Switch
              checked={silentStart}
              onCheckedChange={handleSilentStartChange}
              disabled={isLoading}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
