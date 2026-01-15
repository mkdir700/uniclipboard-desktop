import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import React, { useState, useEffect, ReactNode } from 'react'
import i18n, { normalizeLanguage, persistLanguage } from '@/i18n'
import { SettingContext, type SettingContextType, type Setting } from '@/types/setting'

// 设置变更事件数据接口
interface SettingChangedEvent {
  settingJson: string
  timestamp: number
}

// 设置提供者属性接口
interface SettingProviderProps {
  children: ReactNode
}

// 设置提供者组件
export const SettingProvider: React.FC<SettingProviderProps> = ({ children }) => {
  const [setting, setSetting] = useState<Setting | null>(null)
  const [loading, setLoading] = useState<boolean>(true)
  const [error, setError] = useState<string | null>(null)

  // 加载设置
  const loadSetting = async () => {
    try {
      setLoading(true)
      // New command returns JSON object directly, no parsing needed
      const settingObj = await invoke<Setting>('get_settings')
      setSetting(settingObj)
      setError(null)
    } catch (err) {
      console.error('加载设置失败:', err)
      setError(`加载设置失败: ${err}`)
    } finally {
      setLoading(false)
    }
  }

  // 保存设置
  const saveSetting = async (newSetting: Setting) => {
    try {
      setLoading(true)
      // New command: update_settings, takes JSON object directly
      await invoke('update_settings', { settings: newSetting })
      setSetting(newSetting)
      setError(null)
    } catch (err) {
      console.error('保存设置失败:', err)
      setError(`保存设置失败: ${err}`)
      throw err // 重新抛出错误，让调用者可以处理
    } finally {
      setLoading(false)
    }
  }

  // 更新整个设置
  const updateSetting = async (newSetting: Setting) => {
    await saveSetting(newSetting)
  }

  // 更新通用设置
  const updateGeneralSetting = async (newGeneralSetting: Partial<Setting['general']>) => {
    if (!setting) return
    const updatedSetting = {
      ...setting,
      general: {
        ...setting.general,
        ...newGeneralSetting,
      },
    }
    await saveSetting(updatedSetting)
  }

  // 更新同步设置
  const updateSyncSetting = async (newSyncSetting: Partial<Setting['sync']>) => {
    if (!setting) return
    const updatedSetting = {
      ...setting,
      sync: {
        ...setting.sync,
        ...newSyncSetting,
      },
    }
    await saveSetting(updatedSetting)
  }

  // 更新安全设置
  const updateSecuritySetting = async (newSecuritySetting: Partial<Setting['security']>) => {
    if (!setting) return
    const updatedSetting = {
      ...setting,
      security: {
        ...setting.security,
        ...newSecuritySetting,
      },
    }
    await saveSetting(updatedSetting)
  }

  // 更新网络设置
  const updateNetworkSetting = async (newNetworkSetting: Partial<Setting['network']>) => {
    if (!setting) return
    const updatedSetting = {
      ...setting,
      network: {
        ...setting.network,
        ...newNetworkSetting,
      },
    }
    await saveSetting(updatedSetting)
  }

  // 更新存储设置
  const updateStorageSetting = async (newStorageSetting: Partial<Setting['storage']>) => {
    if (!setting) return
    const updatedSetting = {
      ...setting,
      storage: {
        ...setting.storage,
        ...newStorageSetting,
      },
    }
    await saveSetting(updatedSetting)
  }

  // 初始加载设置
  useEffect(() => {
    loadSetting()
  }, [])

  // 监听来自其他窗口的设置变更事件
  useEffect(() => {
    let unlisten: (() => void) | undefined

    const setupSettingChangeListener = async () => {
      try {
        unlisten = await listen<SettingChangedEvent>('setting-changed', event => {
          console.log('收到设置变更事件:', event.payload)

          // 解析新的设置
          try {
            const newSetting = JSON.parse(event.payload.settingJson) as Setting

            // 更新本地状态 (不触发再次保存)
            setSetting(newSetting)
          } catch (err) {
            console.error('解析设置变更事件失败:', err)
          }
        })
      } catch (err) {
        console.error('设置设置变更监听器失败:', err)
      }
    }

    setupSettingChangeListener()

    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  }, [])

  // 监听主题变化并应用
  useEffect(() => {
    const root = window.document.documentElement
    const systemThemeMedia = window.matchMedia('(prefers-color-scheme: dark)')

    const applyTheme = () => {
      const theme = setting?.general.theme
      const themeColor = setting?.general.theme_color || 'catppuccin'

      // 1. Apply Mode (Light/Dark)
      root.classList.remove('light', 'dark')

      if (theme === 'system' || !theme) {
        const systemTheme = systemThemeMedia.matches ? 'dark' : 'light'
        root.classList.add(systemTheme)
      } else {
        root.classList.add(theme)
      }

      // 2. Apply Theme Color
      root.setAttribute('data-theme', themeColor)
    }

    applyTheme()

    const handleSystemThemeChange = () => {
      if (setting?.general.theme === 'system' || !setting?.general.theme) {
        applyTheme()
      }
    }

    systemThemeMedia.addEventListener('change', handleSystemThemeChange)

    return () => {
      systemThemeMedia.removeEventListener('change', handleSystemThemeChange)
    }
  }, [setting?.general.theme, setting?.general.theme_color])

  // 监听语言变化并应用
  useEffect(() => {
    const next = normalizeLanguage(setting?.general?.language)
    if (i18n.language !== next) {
      i18n.changeLanguage(next)
    }
    persistLanguage(next)
  }, [setting?.general?.language])

  const value: SettingContextType = {
    setting,
    loading,
    error,
    updateSetting,
    updateGeneralSetting,
    updateSyncSetting,
    updateSecuritySetting,
    updateNetworkSetting,
    updateStorageSetting,
  }

  return <SettingContext.Provider value={value}>{children}</SettingContext.Provider>
}

export { SettingContext }
