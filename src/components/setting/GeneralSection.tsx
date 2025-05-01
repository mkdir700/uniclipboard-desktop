import { useState, useEffect } from "react";
import Toggle from "../ui/Toggle";
import { useSetting } from "../../contexts/SettingContext";
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
  const handleAutoStartChange = async () => {
    try {
      setIsLoading(true);
      const newState = !autoStart;

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
        <Toggle
          checked={autoStart}
          onChange={handleAutoStartChange}
          label="开机自启动"
          description="系统启动时自动启动Uniclipboard"
          disabled={isLoading}
        />
      </div>
    </>
  );
}
