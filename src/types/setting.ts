import { createContext } from 'react'

// 内容类型接口
interface ContentTypes {
  text: boolean
  image: boolean
  link: boolean
  file: boolean
  code_snippet: boolean
  rich_text: boolean
}

// 主题模式类型
export type ThemeMode = 'light' | 'dark' | 'system'

// 通用设置接口
export interface GeneralSetting {
  auto_start: boolean
  silent_start: boolean
  auto_check_update: boolean
  theme: ThemeMode
  theme_color: string
  language: string
  device_name?: string
}

// 同步设置接口
export interface SyncSetting {
  auto_sync: boolean
  sync_frequency: string
  content_types: ContentTypes
  max_file_size: number
}

// 安全设置接口
export interface SecuritySetting {
  end_to_end_encryption: boolean
  password: string
}

// 网络设置接口
export interface NetworkSetting {
  sync_method: string
  cloud_server: string
  webserver_port: number
  custom_peer_device: boolean
  peer_device_addr: string | null
  peer_device_port: number | null
}

// 存储设置接口
export interface StorageSetting {
  auto_clear_history: string
  history_retention_days: number
  max_history_items: number
}

// 关于设置接口
export interface AboutSetting {
  version: string
}

// 设置接口
export interface Setting {
  general: GeneralSetting
  sync: SyncSetting
  security: SecuritySetting
  network: NetworkSetting
  storage: StorageSetting
  about: AboutSetting
}

// 设置上下文接口
export interface SettingContextType {
  setting: Setting | null
  loading: boolean
  error: string | null
  updateSetting: (newSetting: Setting) => Promise<void>
  updateGeneralSetting: (newGeneralSetting: Partial<GeneralSetting>) => Promise<void>
  updateSyncSetting: (newSyncSetting: Partial<SyncSetting>) => Promise<void>
  updateSecuritySetting: (newSecuritySetting: Partial<SecuritySetting>) => Promise<void>
  updateNetworkSetting: (newNetworkSetting: Partial<NetworkSetting>) => Promise<void>
  updateStorageSetting: (newStorageSetting: Partial<StorageSetting>) => Promise<void>
}

// 创建设置上下文
export const SettingContext = createContext<SettingContextType | undefined>(undefined)
