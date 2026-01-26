import { createContext } from 'react'
import type { SettingContextType } from '@/types/setting'

export const SettingContext = createContext<SettingContextType | undefined>(undefined)
