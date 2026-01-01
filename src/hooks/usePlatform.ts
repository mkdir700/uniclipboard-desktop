import { useMemo } from 'react'

/**
 * 检查是否在 Tauri 环境中运行
 */
const isTauriEnv = () =>
  typeof window !== 'undefined' &&
  Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__)

/**
 * 平台检测结果接口
 */
export interface PlatformInfo {
  /** 是否为 Windows 平台 */
  isWindows: boolean
  /** 是否为 macOS 平台 */
  isMac: boolean
  /** 是否为 Linux 平台 */
  isLinux: boolean
  /** 是否在 Tauri 环境中运行 */
  isTauri: boolean
}

/**
 * 平台检测 Hook
 *
 * 提供集中的平台检测功能，用于实现跨平台的条件渲染和行为控制。
 *
 * @example
 * ```tsx
 * const { isWindows, isMac, isTauri } = usePlatform()
 *
 * if (isWindows && isTauri) {
 *   // Windows 特定逻辑
 * }
 * ```
 */
export const usePlatform = (): PlatformInfo => {
  return useMemo(() => {
    // 检查是否在 Tauri 环境
    const isTauri = isTauriEnv()

    // 如果不在浏览器环境，返回默认值
    if (typeof navigator === 'undefined') {
      return {
        isWindows: false,
        isMac: false,
        isLinux: false,
        isTauri,
      }
    }

    // 获取平台信息
    const userAgent = navigator.userAgent.toLowerCase()
    const platform = (
      navigator as unknown as { userAgentData?: { platform?: string } }
    ).userAgentData?.platform?.toLowerCase()

    // 检测 Windows
    const isWindows = userAgent.includes('windows') || platform === 'windows'

    // 检测 macOS
    const isMac = userAgent.includes('mac') || platform === 'mac'

    // 检测 Linux (排除 Android)
    const isLinux =
      (userAgent.includes('linux') || platform === 'linux') && !userAgent.includes('android')

    return {
      isWindows,
      isMac,
      isLinux,
      isTauri,
    }
    // 依赖项为空，确保只在组件挂载时计算一次
  }, [])
}
