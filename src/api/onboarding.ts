import { invoke } from '@tauri-apps/api/core'

export interface OnboardingStatus {
  has_completed: boolean
  encryption_password_set: boolean
  device_registered: boolean
}

/**
 * Get current onboarding state
 * 获取当前入门引导状态
 */
export async function getOnboardingState(): Promise<OnboardingStatus> {
  return await invoke('get_onboarding_state')
}

/**
 * Initialize onboarding and get initial state
 * 初始化入门引导并获取初始状态
 */
export async function initializeOnboarding(): Promise<OnboardingStatus> {
  return await invoke('initialize_onboarding')
}

/**
 * Complete onboarding
 * 完成入门引导流程
 */
export async function completeOnboarding(): Promise<void> {
  return await invoke('complete_onboarding')
}

/**
 * @deprecated Use getOnboardingState instead
 * 检查 onboarding 状态
 * 返回详细的状态信息，包括是否已完成、vault 是否初始化、设备是否注册、加密密码是否设置
 */
export async function checkOnboardingStatus(): Promise<OnboardingStatus> {
  return await getOnboardingState()
}

/**
 * @deprecated Use initialize_encryption command instead
 * 设置加密密码（onboarding 专用）
 * 验证密码强度，并确保密码未设置过
 */
export async function setupEncryptionPassword(password: string): Promise<void> {
  console.log(
    '[setupEncryptionPassword] Calling initialize_encryption with password length:',
    password.length
  )
  try {
    await invoke('initialize_encryption', { passphrase: password })
    console.log('[setupEncryptionPassword] initialize_encryption succeeded')
  } catch (error) {
    console.error('[setupEncryptionPassword] initialize_encryption failed:', error)
    throw error
  }
}

export interface DeviceInfo {
  alias?: string
  platform?: string
}

export async function getDeviceId(): Promise<string> {
  return await invoke('get_device_id')
}

export async function saveDeviceInfo(info: DeviceInfo): Promise<void> {
  return await invoke('save_device_info', { info })
}
