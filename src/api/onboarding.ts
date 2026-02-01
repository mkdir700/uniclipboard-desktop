import { invokeWithTrace } from '@/lib/tauri-command'

export interface OnboardingStatus {
  has_completed: boolean
  encryption_password_set: boolean
  device_registered: boolean
}

export type SetupError =
  | 'PassphraseMismatch'
  | { PassphraseTooShort: { min_len: number } }
  | 'PassphraseEmpty'
  | 'PassphraseInvalidOrMismatch'
  | 'NetworkTimeout'
  | 'PeerUnavailable'
  | 'PairingRejected'
  | 'PairingFailed'

export type SetupState =
  | 'Welcome'
  | 'Done'
  | { CreateSpacePassphrase: { error: SetupError | null } }
  | { JoinSpacePickDevice: { error: SetupError | null } }
  | { JoinSpaceVerifyPassphrase: { peer_id: string; error: SetupError | null } }
  | {
      PairingConfirm: {
        session_id: string
        short_code: string
        peer_fingerprint?: string | null
        error: SetupError | null
      }
    }
  | { JoinSpaceKeyslotReceived: { peer_id: string; error: SetupError | null } }

export type SetupEvent =
  | 'ChooseCreateSpace'
  | 'ChooseJoinSpace'
  | 'Back'
  | { SubmitCreatePassphrase: { pass1: string; pass2: string } }
  | { SelectPeer: { peer_id: string } }
  | { SubmitJoinPassphrase: { passphrase: string } }
  | 'PairingUserConfirm'
  | 'PairingUserCancel'
  | 'PairingSucceeded'
  | { PairingFailed: { reason: SetupError } }
  | { KeyslotReceived: { peer_id: string } }
  | 'ChallengeVerified'
  | 'PassphraseMismatch'
  | 'NetworkScanRefresh'

/**
 * Get current onboarding state
 * 获取当前入门引导状态
 */
export async function getOnboardingState(): Promise<OnboardingStatus> {
  return await invokeWithTrace('get_onboarding_state')
}

/**
 * Initialize onboarding and get initial state
 * 初始化入门引导并获取初始状态
 */
export async function initializeOnboarding(): Promise<OnboardingStatus> {
  return await invokeWithTrace('initialize_onboarding')
}

/**
 * Complete onboarding
 * 完成入门引导流程
 */
export async function completeOnboarding(): Promise<void> {
  return await invokeWithTrace('complete_onboarding')
}

/**
 * Get current setup state
 * 获取当前设置流程状态
 */
export async function getSetupState(): Promise<SetupState> {
  return await invokeWithTrace('get_setup_state')
}

/**
 * Dispatch a setup event
 * 分发设置事件
 */
export async function dispatchSetupEvent(event: SetupEvent): Promise<SetupState> {
  return await invokeWithTrace('dispatch_setup_event', { event })
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
    await invokeWithTrace('initialize_encryption', { passphrase: password })
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
  return await invokeWithTrace('get_device_id')
}

export async function saveDeviceInfo(info: DeviceInfo): Promise<void> {
  return await invokeWithTrace('save_device_info', { info })
}
