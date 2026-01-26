import type { Update } from '@tauri-apps/plugin-updater'
import { createContext } from 'react'

export interface UpdateContextType {
  updateInfo: Update | null
  isCheckingUpdate: boolean
  checkForUpdates: () => Promise<Update | null>
}

export const UpdateContext = createContext<UpdateContextType | undefined>(undefined)
