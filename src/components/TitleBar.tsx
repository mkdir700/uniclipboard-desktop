import { enableModernWindowStyle } from '@cloudworxx/tauri-plugin-mac-rounded-corners'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Minus, Square, X, ArrowLeft, Search } from 'lucide-react'
import React, { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useLocation } from 'react-router-dom'
import { Input } from '@/components/ui/input'
import { usePlatform } from '@/hooks/usePlatform'
import { cn } from '@/lib/utils'

interface TitleBarProps {
  className?: string
  searchValue?: string
  onSearchChange?: (value: string) => void
}

// macOS window style configuration (must match enableModernWindowStyle call)
const MAC_WINDOW_STYLE = {
  cornerRadius: 12,
  offsetX: -15,
  offsetY: -3,
} as const

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
    onClick={e => {
      console.log('[TitleBarButton] Button clicked:', ariaLabel)
      e.stopPropagation()
      onClick()
    }}
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

export const TitleBar = ({ className, searchValue = '', onSearchChange }: TitleBarProps) => {
  const [isMaximized, setIsMaximized] = useState(false)
  const navigate = useNavigate()
  const location = useLocation()
  const { t } = useTranslation()

  // 使用 usePlatform hook 获取平台信息
  const { isWindows, isMac, isTauri } = usePlatform()
  const windowRef = useMemo(() => (isTauri ? getCurrentWindow() : null), [isTauri])

  // 检测是否在 Settings 页面
  const isSettingsPage = location.pathname.startsWith('/settings')
  // 检测是否在 Dashboard 页面
  const isDashboardPage = location.pathname === '/'

  useEffect(() => {
    if (!isTauri || !windowRef) return

    let mounted = true

    // Enable macOS rounded corners
    if (isMac) {
      enableModernWindowStyle(MAC_WINDOW_STYLE).catch(error => {
        console.error('[TitleBar] Failed to enable rounded corners:', error)
      })
    }

    windowRef.isMaximized().then(value => {
      if (mounted) setIsMaximized(value)
    })

    const unlistenPromise = windowRef.onResized(async () => {
      if (!mounted) return
      setIsMaximized(await windowRef.isMaximized())
    })

    return () => {
      mounted = false
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [isTauri, isMac, windowRef])

  const handleMinimize = async () => {
    console.log('[TitleBar] Minimize clicked, isTauri:', isTauri)
    if (!isTauri || !windowRef) return
    try {
      console.log('[TitleBar] Calling minimize...')
      await windowRef.minimize()
      console.log('[TitleBar] Minimize succeeded')
    } catch (error) {
      console.error('[TitleBar] Minimize failed:', error)
    }
  }

  const handleToggleMaximize = async () => {
    console.log('[TitleBar] Toggle maximize clicked, isTauri:', isTauri)
    if (!isTauri || !windowRef) return
    try {
      const maximized = await windowRef.isMaximized()
      console.log('[TitleBar] Current maximized state:', maximized)
      if (maximized) {
        await windowRef.unmaximize()
      } else {
        await windowRef.maximize()
      }
      setIsMaximized(!maximized)
      console.log('[TitleBar] Toggle maximize succeeded')
    } catch (error) {
      console.error('[TitleBar] Toggle maximize failed:', error)
    }
  }

  const handleClose = async () => {
    console.log('[TitleBar] Close clicked, isTauri:', isTauri)
    if (!isTauri || !windowRef) return
    try {
      console.log('[TitleBar] Calling close...')
      await windowRef.close()
      console.log('[TitleBar] Close succeeded')
    } catch (error) {
      console.error('[TitleBar] Close failed:', error)
    }
  }

  const handleBack = () => {
    navigate(-1)
  }

  const [isSearchFocused, setIsSearchFocused] = useState(false)

  return (
    <div
      className={cn(
        'fixed top-0 left-0 right-0 h-10 z-50 select-none',
        'bg-background/70 backdrop-blur border-b border-border/60',
        className
      )}
    >
      <div className="h-full flex items-center justify-between cursor-default">
        <div
          data-tauri-drag-region
          className={cn(
            'flex-1 flex items-center',
            'pr-4',
            // On macOS, add left padding to avoid traffic lights
            // On other platforms, use default padding
            isMac ? `pl-16` : 'px-3',
            isDashboardPage ? 'justify-center' : ''
          )}
          onDoubleClick={isWindows ? handleToggleMaximize : undefined}
        >
          {isSettingsPage ? (
            <button
              onClick={handleBack}
              data-tauri-drag-region="false"
              className="flex items-center justify-center rounded-md p-1.5 text-muted-foreground transition-colors hover:bg-muted/70 hover:text-foreground active:bg-primary/10 active:text-primary"
              aria-label={t('nav.back')}
            >
              <ArrowLeft className="h-4 w-4" />
            </button>
          ) : isDashboardPage ? (
            <div
              className={cn(
                'relative flex items-center w-64 max-w-xs',
                'transition-all duration-200'
              )}
            >
              <Search
                className={cn(
                  'absolute left-2.5 h-3.5 w-3.5 transition-colors duration-200',
                  isSearchFocused ? 'text-primary' : 'text-muted-foreground'
                )}
              />
              <Input
                data-tauri-drag-region="false"
                type="text"
                value={searchValue}
                onChange={e => onSearchChange?.(e.target.value)}
                placeholder={t('header.searchPlaceholder')}
                className={cn(
                  'h-7 w-full pl-8 pr-2.5 py-1',
                  'bg-muted/50 hover:bg-muted/70',
                  'border border-border/50 rounded-lg text-sm',
                  'focus-visible:bg-background focus-visible:border-primary/50',
                  'transition-all duration-200',
                  'focus-visible:ring-0 focus-visible:ring-offset-0',
                  'placeholder:text-muted-foreground/50'
                )}
                onFocus={() => setIsSearchFocused(true)}
                onBlur={() => setIsSearchFocused(false)}
              />
            </div>
          ) : null}
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
