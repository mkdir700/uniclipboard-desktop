import { invoke } from '@tauri-apps/api/core'

export interface OnboardingStatus {
  has_completed: boolean
  vault_initialized: boolean
  device_registered: boolean
  encryption_password_set: boolean
}

/**
 * 检查 onboarding 状态
 * 返回详细的状态信息，包括是否已完成、vault 是否初始化、设备是否注册、加密密码是否设置
 */
export async function checkOnboardingStatus(): Promise<OnboardingStatus> {
  return await invoke('check_onboarding_status')
}

/**
 * 完成 onboarding 流程
 * 会验证 vault 密钥、加密密码和设备注册状态，然后创建完成标记
 */
export async function completeOnboarding(): Promise<void> {
  return await invoke('complete_onboarding')
}

/**
 * 设置加密密码（onboarding 专用）
 * 验证密码强度，并确保密码未设置过
 */
export async function setupEncryptionPassword(password: string): Promise<void> {
  return await invoke('setup_encryption_password', { password })
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
