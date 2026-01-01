import { invoke } from '@tauri-apps/api/core'

export interface VaultStatus {
  is_initialized: boolean
}

export async function checkVaultStatus(): Promise<VaultStatus> {
  return await invoke('check_vault_status')
}

export async function resetVault(): Promise<void> {
  return await invoke('reset_vault')
}
