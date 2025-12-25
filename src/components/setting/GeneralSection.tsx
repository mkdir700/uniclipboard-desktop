import { useState, useEffect } from "react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Switch } from "@/components/ui";
import { useSetting } from "@/contexts/SettingContext";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { SUPPORTED_LANGUAGES, type SupportedLanguage } from "@/i18n";

export default function GeneralSection() {
  const { t } = useTranslation();
  const { setting, updateGeneralSetting } = useSetting();
  const [autoStart, setAutoStart] = useState(false);
  const [silentStart, setSilentStart] = useState(false);
  const [language, setLanguage] = useState<SupportedLanguage>("zh-CN");
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
          setLanguage((setting.general.language as SupportedLanguage) || "zh-CN");
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

  const handleLanguageChange = async (next: string) => {
    try {
      const normalized = (next as SupportedLanguage) || "zh-CN";
      await updateGeneralSetting({ language: normalized });
      setLanguage(normalized);
    } catch (error) {
      console.error("更改语言失败:", error);
    }
  };

  return (
    <div className="space-y-6">
      <div className="space-y-4">
        <h4 className="text-base font-medium px-2">
          {t("settings.sections.general.startupTitle")}
        </h4>

        <div className="py-2 rounded-lg px-2">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <h4 className="text-sm font-medium">
                {t("settings.sections.general.autoStart.label")}
              </h4>
              <p className="text-xs text-muted-foreground">
                {t("settings.sections.general.autoStart.description")}
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
              <h4 className="text-sm font-medium">
                {t("settings.sections.general.silentStart.label")}
              </h4>
              <p className="text-xs text-muted-foreground">
                {t("settings.sections.general.silentStart.description")}
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

      <div className="space-y-4">
        <h4 className="text-base font-medium px-2">
          {t("settings.sections.general.language.title")}
        </h4>

        <div className="py-2 rounded-lg px-2">
          <div className="flex items-center justify-between gap-4">
            <div className="space-y-0.5">
              <h4 className="text-sm font-medium">
                {t("settings.sections.general.language.label")}
              </h4>
              <p className="text-xs text-muted-foreground">
                {t("settings.sections.general.language.description")}
              </p>
            </div>

            <div className="w-40">
              <Select
                value={language}
                onValueChange={handleLanguageChange}
                disabled={isLoading}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {SUPPORTED_LANGUAGES.map((lang) => (
                    <SelectItem key={lang} value={lang}>
                      {t(`language.${lang}`)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
