import { check, type Update } from '@tauri-apps/plugin-updater'
import React, { createContext, useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from '@/components/ui/sonner'
import { useSetting } from '@/hooks/useSetting'

export interface UpdateContextType {
  updateInfo: Update | null
  isCheckingUpdate: boolean
  checkForUpdates: () => Promise<Update | null>
}

export const UpdateContext = createContext<UpdateContextType | undefined>(undefined)

interface UpdateProviderProps {
  children: React.ReactNode
}

export const UpdateProvider: React.FC<UpdateProviderProps> = ({ children }) => {
  const { t } = useTranslation()
  const { setting } = useSetting()
  const [updateInfo, setUpdateInfo] = useState<Update | null>(null)
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false)
  const isCheckingRef = useRef(false)
  const hasCheckedOnStartup = useRef(false)

  const checkForUpdates = useCallback(async () => {
    if (isCheckingRef.current) {
      return updateInfo
    }

    isCheckingRef.current = true
    setIsCheckingUpdate(true)

    try {
      const update = await check()
      setUpdateInfo(update)
      return update
    } finally {
      isCheckingRef.current = false
      setIsCheckingUpdate(false)
    }
  }, [updateInfo])

  useEffect(() => {
    if (!setting?.general || hasCheckedOnStartup.current) {
      return
    }

    hasCheckedOnStartup.current = true

    if (!setting.general.auto_check_update) {
      return
    }

    checkForUpdates().catch(error => {
      console.error('检查更新失败:', error)
      toast.error(t('update.checkFailed'))
    })
  }, [setting?.general, checkForUpdates, t])

  const value = useMemo(
    () => ({
      updateInfo,
      isCheckingUpdate,
      checkForUpdates,
    }),
    [updateInfo, isCheckingUpdate, checkForUpdates]
  )

  return <UpdateContext.Provider value={value}>{children}</UpdateContext.Provider>
}
