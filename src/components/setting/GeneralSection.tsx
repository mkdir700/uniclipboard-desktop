import { useState, useEffect } from "react";
import { Switch } from "@/components/ui";
import { useSetting } from "@/contexts/SettingContext";
import { invoke } from "@tauri-apps/api/core";

export default function GeneralSection() {
  const { setting, updateGeneralSetting } = useSetting();
  const [autoStart, setAutoStart] = useState(false);
  const [silentStart, setSilentStart] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  // 初始化时检查自启动状态和设置
  useEffect(() => {
    const checkStatus = async () => {
      try {
        setIsLoading(true);
        // 检查系统自启动状态
        const enabled = await invoke("is_autostart_enabled");
        setAutoStart(enabled as boolean);
        
        // 从配置中读取静默启动状态
        if (setting?.general) {
          setSilentStart(setting.general.silent_start);
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

  return (
    <div className="space-y-4">
      <div className="py-2 rounded-lg px-2">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h4 className="text-base font-medium">开机自启动</h4>
            <p className="text-sm text-muted-foreground">
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
            <h4 className="text-base font-medium">静默启动</h4>
            <p className="text-sm text-muted-foreground">
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
  );
}
