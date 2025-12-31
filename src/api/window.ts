import { invoke } from '@tauri-apps/api/core'

/**
 * Opens the settings window in a new independent window.
 * If the settings window is already open, it will be focused.
 */
export async function openSettingsWindow(): Promise<void> {
  return invoke<void>('open_settings_window')
}
