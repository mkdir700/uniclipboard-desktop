import { getCurrentWindow } from '@tauri-apps/api/window'
import { Minus, Square, X, ArrowLeft } from 'lucide-react'
import React, { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useLocation } from 'react-router-dom'
import { cn } from '@/lib/utils'

interface TitleBarProps {
  className?: string
}

const isTauriEnv = () =>
  typeof window !== 'undefined' &&
  Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__)

const isWindowsPlatform = () => {
  if (typeof navigator === 'undefined') return false
  const userAgent = navigator.userAgent.toLowerCase()
  const platform = (navigator as unknown as { userAgentData?: { platform?: string } }).userAgentData
    ?.platform

  return userAgent.includes('windows') || platform?.toLowerCase() === 'windows'
}

const TitleBarButton = ({
  onClick,
  children,
  className,
  'aria-label': ariaLabel,
}: {
  onClick: () => void
  children: React.ReactNode
  className?: string
  'aria-label': string
}) => (
  <button
    type="button"
    aria-label={ariaLabel}
    data-tauri-drag-region="false"
    onClick={onClick}
    onDoubleClick={event => event.stopPropagation()}
    className={cn(
      'h-full w-12 flex items-center justify-center transition-colors duration-150',
      'text-muted-foreground hover:text-foreground',
      className
    )}
  >
    {children}
  </button>
)

export const TitleBar = ({ className }: TitleBarProps) => {
  const [isWindows, setIsWindows] = useState(false)
  const [isMaximized, setIsMaximized] = useState(false)
  const navigate = useNavigate()
  const location = useLocation()
  const { t } = useTranslation()

  const isTauri = useMemo(() => isTauriEnv(), [])

  // 检测是否在 Settings 页面
  const isSettingsPage = location.pathname.startsWith('/settings')

  useEffect(() => {
    if (!isTauri) return

    let mounted = true
    const currentWindow = getCurrentWindow()

    setIsWindows(isWindowsPlatform())

    currentWindow.isMaximized().then(value => {
      if (mounted) setIsMaximized(value)
    })

    const unlistenPromise = currentWindow.onResized(async () => {
      if (!mounted) return
      setIsMaximized(await currentWindow.isMaximized())
    })

    return () => {
      mounted = false
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [isTauri])

  const handleMinimize = async () => {
    if (!isTauri) return
    await getCurrentWindow().minimize()
  }

  const handleToggleMaximize = async () => {
    if (!isTauri) return
    const window = getCurrentWindow()
    const maximized = await window.isMaximized()
    if (maximized) {
      await window.unmaximize()
    } else {
      await window.maximize()
    }
    setIsMaximized(!maximized)
  }

  const handleClose = async () => {
    if (!isTauri) return
    await getCurrentWindow().close()
  }

  const handleBack = () => {
    navigate(-1)
  }

  return (
    <div
      className={cn(
        'fixed top-0 left-0 right-0 h-10 z-50 select-none',
        'bg-background/70 backdrop-blur border-b border-border/60',
        className
      )}
    >
      <div
        className="h-full flex items-center justify-between cursor-default"
      >
        <div
          data-tauri-drag-region
          className="flex-1 flex items-center px-3"
          onDoubleClick={isWindows ? handleToggleMaximize : undefined}
        >
          {isSettingsPage ? (
            <button
              onClick={handleBack}
              data-tauri-drag-region="false"
              className="flex items-center justify-center rounded-md p-1.5 text-muted-foreground transition-colors hover:bg-muted/70 hover:text-foreground active:bg-primary/10 active:text-primary"
              aria-label={t("nav.back")}
            >
              <ArrowLeft className="h-4 w-4" />
            </button>
          ) : (
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <div className="w-2 h-2 rounded-full bg-primary/80 shadow-sm" />
              <span className="font-medium text-foreground">UniClipboard</span>
            </div>
          )}
        </div>
        {isWindows && (
          <div className="flex items-center h-full" data-tauri-drag-region="false">
            <TitleBarButton aria-label="最小化" onClick={handleMinimize}>
              <Minus className="h-4 w-4" />
            </TitleBarButton>
            <TitleBarButton
              aria-label={isMaximized ? '还原' : '最大化'}
              onClick={handleToggleMaximize}
            >
              <Square className="h-3.5 w-3.5" />
            </TitleBarButton>
            <TitleBarButton
              aria-label="关闭"
              onClick={handleClose}
              className="hover:bg-red-500/90 hover:text-white"
            >
              <X className="h-4 w-4" />
            </TitleBarButton>
          </div>
        )}
      </div>
    </div>
  )
}

export default TitleBar
