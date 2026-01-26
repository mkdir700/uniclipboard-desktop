import { invokeWithTrace } from '@/lib/tauri-command'

export interface VaultStatus {
  is_initialized: boolean
}

export async function checkVaultStatus(): Promise<VaultStatus> {
  return await invokeWithTrace('check_vault_status')
}

export async function resetVault(): Promise<void> {
  return await invokeWithTrace('reset_vault')
}
