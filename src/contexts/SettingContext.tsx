import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import i18n, { normalizeLanguage, persistLanguage } from "@/i18n";

// 内容类型接口
interface ContentTypes {
  text: boolean;
  image: boolean;
  link: boolean;
  file: boolean;
  code_snippet: boolean;
  rich_text: boolean;
}

// 主题模式类型
export type ThemeMode = "light" | "dark" | "system";

// 通用设置接口
interface GeneralSetting {
  auto_start: boolean;
  silent_start: boolean;
  auto_check_update: boolean;
  theme: ThemeMode;
  theme_color: string;
  language: string;
}

// 同步设置接口
interface SyncSetting {
  auto_sync: boolean;
  sync_frequency: string;
  content_types: ContentTypes;
  max_file_size: number;
}

// 安全设置接口
interface SecuritySetting {
  end_to_end_encryption: boolean;
  password: string;
}

// 网络设置接口
interface NetworkSetting {
  sync_method: string;
  cloud_server: string;
  webserver_port: number;
  custom_peer_device: boolean;
  peer_device_addr: string | null;
  peer_device_port: number | null;
}

// 存储设置接口
interface StorageSetting {
  auto_clear_history: string;
  history_retention_days: number;
  max_history_items: number;
}

// 关于设置接口
interface AboutSetting {
  version: string;
}

// 设置接口
export interface Setting {
  general: GeneralSetting;
  sync: SyncSetting;
  security: SecuritySetting;
  network: NetworkSetting;
  storage: StorageSetting;
  about: AboutSetting;
}

// 设置上下文接口
interface SettingContextType {
  setting: Setting | null;
  loading: boolean;
  error: string | null;
  updateSetting: (newSetting: Setting) => Promise<void>;
  updateGeneralSetting: (
    newGeneralSetting: Partial<GeneralSetting>
  ) => Promise<void>;
  updateSyncSetting: (newSyncSetting: Partial<SyncSetting>) => Promise<void>;
  updateSecuritySetting: (
    newSecuritySetting: Partial<SecuritySetting>
  ) => Promise<void>;
  updateNetworkSetting: (
    newNetworkSetting: Partial<NetworkSetting>
  ) => Promise<void>;
  updateStorageSetting: (
    newStorageSetting: Partial<StorageSetting>
  ) => Promise<void>;
}

// 创建设置上下文
const SettingContext = createContext<SettingContextType | undefined>(undefined);

// 设置提供者属性接口
interface SettingProviderProps {
  children: ReactNode;
}

// 设置提供者组件
export const SettingProvider: React.FC<SettingProviderProps> = ({
  children,
}) => {
  const [setting, setSetting] = useState<Setting | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  // 加载设置
  const loadSetting = async () => {
    try {
      setLoading(true);
      // 直接获取设置并转换为Setting对象
      const result = await invoke<string>("get_setting");
      const settingObj = JSON.parse(result) as Setting;
      setSetting(settingObj);
      setError(null);
    } catch (err) {
      console.error("加载设置失败:", err);
      setError(`加载设置失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  // 保存设置
  const saveSetting = async (newSetting: Setting) => {
    try {
      setLoading(true);
      await invoke("save_setting", { settingJson: JSON.stringify(newSetting) });
      setSetting(newSetting);
      setError(null);
    } catch (err) {
      console.error("保存设置失败:", err);
      setError(`保存设置失败: ${err}`);
      throw err; // 重新抛出错误，让调用者可以处理
    } finally {
      setLoading(false);
    }
  };

  // 更新整个设置
  const updateSetting = async (newSetting: Setting) => {
    await saveSetting(newSetting);
  };

  // 更新通用设置
  const updateGeneralSetting = async (
    newGeneralSetting: Partial<GeneralSetting>
  ) => {
    if (!setting) return;
    const updatedSetting = {
      ...setting,
      general: {
        ...setting.general,
        ...newGeneralSetting,
      },
    };
    await saveSetting(updatedSetting);
  };

  // 更新同步设置
  const updateSyncSetting = async (newSyncSetting: Partial<SyncSetting>) => {
    if (!setting) return;
    const updatedSetting = {
      ...setting,
      sync: {
        ...setting.sync,
        ...newSyncSetting,
      },
    };
    await saveSetting(updatedSetting);
  };

  // 更新安全设置
  const updateSecuritySetting = async (
    newSecuritySetting: Partial<SecuritySetting>
  ) => {
    if (!setting) return;
    const updatedSetting = {
      ...setting,
      security: {
        ...setting.security,
        ...newSecuritySetting,
      },
    };
    await saveSetting(updatedSetting);
  };

  // 更新网络设置
  const updateNetworkSetting = async (
    newNetworkSetting: Partial<NetworkSetting>
  ) => {
    if (!setting) return;
    const updatedSetting = {
      ...setting,
      network: {
        ...setting.network,
        ...newNetworkSetting,
      },
    };
    await saveSetting(updatedSetting);
  };

  // 更新存储设置
  const updateStorageSetting = async (
    newStorageSetting: Partial<StorageSetting>
  ) => {
    if (!setting) return;
    const updatedSetting = {
      ...setting,
      storage: {
        ...setting.storage,
        ...newStorageSetting,
      },
    };
    await saveSetting(updatedSetting);
  };

  // 初始加载设置
  useEffect(() => {
    loadSetting();
  }, []);

  // 监听主题变化并应用
  useEffect(() => {
    const root = window.document.documentElement;
    const systemThemeMedia = window.matchMedia("(prefers-color-scheme: dark)");

    const applyTheme = () => {
      const theme = setting?.general.theme;
      const themeColor = setting?.general.theme_color || "catppuccin";

      // 1. Apply Mode (Light/Dark)
      root.classList.remove("light", "dark");

      if (theme === "system" || !theme) {
        const systemTheme = systemThemeMedia.matches ? "dark" : "light";
        root.classList.add(systemTheme);
      } else {
        root.classList.add(theme);
      }

      // 2. Apply Theme Color
      root.setAttribute("data-theme", themeColor);
    };

    applyTheme();

    const handleSystemThemeChange = () => {
      if (setting?.general.theme === "system" || !setting?.general.theme) {
        applyTheme();
      }
    };

    systemThemeMedia.addEventListener("change", handleSystemThemeChange);

    return () => {
      systemThemeMedia.removeEventListener("change", handleSystemThemeChange);
    };
  }, [setting?.general.theme, setting?.general.theme_color]);

  // 监听语言变化并应用
  useEffect(() => {
    const next = normalizeLanguage(setting?.general?.language);
    if (i18n.language !== next) {
      i18n.changeLanguage(next);
    }
    persistLanguage(next);
  }, [setting?.general?.language]);

  return (
    <SettingContext.Provider
      value={{
        setting,
        loading,
        error,
        updateSetting,
        updateGeneralSetting,
        updateSyncSetting,
        updateSecuritySetting,
        updateNetworkSetting,
        updateStorageSetting,
      }}
    >
      {children}
    </SettingContext.Provider>
  );
};

// 使用设置上下文的钩子
export const useSetting = () => {
  const context = useContext(SettingContext);
  if (context === undefined) {
    throw new Error("useSetting必须在SettingProvider内部使用");
  }
  return context;
};
