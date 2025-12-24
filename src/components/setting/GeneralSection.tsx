import { useState, useEffect } from "react";
import { Switch } from "@/components/ui";
import { useSetting } from "@/contexts/SettingContext";
import { invoke } from "@tauri-apps/api/core";

export default function GeneralSection() {
  const { updateGeneralSetting } = useSetting();
  const [autoStart, setAutoStart] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  // 初始化时检查自启动状态
  useEffect(() => {
    const checkAutoStartStatus = async () => {
      try {
        setIsLoading(true);
        const enabled = await invoke("is_autostart_enabled");
        setAutoStart(enabled as boolean);
      } catch (error) {
        console.error("检查自启动状态失败:", error);
      } finally {
        setIsLoading(false);
      }
    };

    checkAutoStartStatus();
  }, []);

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

  return (
    <>
      <div className="settings-item py-2 rounded-lg px-2">
        <div className="flex items-center justify-between">
          <div className="flex-1">
            <h4 className="text-sm font-medium text-white">开机自启动</h4>
            <p className="text-xs text-gray-400 mt-0.5">
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
    </>
  );
}
