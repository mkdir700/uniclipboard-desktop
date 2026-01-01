import { enableModernWindowStyle } from '@cloudworxx/tauri-plugin-mac-rounded-corners'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { ArrowLeft, Search } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
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

export const TitleBar = ({ className, searchValue = '', onSearchChange }: TitleBarProps) => {
  const [isMaximized, setIsMaximized] = useState(false) // Used for macOS double-click maximize
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

  // Windows: Double-click to maximize is handled by decorum
  // macOS: We handle it manually for the drag region
  const handleToggleMaximize = async () => {
    if (isWindows) return // decorum handles this on Windows
    if (!isTauri || !windowRef) return
    try {
      const maximized = isMaximized // Use current state value
      if (maximized) {
        await windowRef.unmaximize()
      } else {
        await windowRef.maximize()
      }
      setIsMaximized(!maximized)
    } catch (error) {
      console.error('[TitleBar] Toggle maximize failed:', error)
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
            // On Windows with decorum, reserve space for the injected controls (right side)
            // On macOS, add left padding to avoid traffic lights
            // On other platforms, use default padding
            isMac ? 'pl-16 pr-4' : isWindows ? 'px-3' : 'px-3 pr-4',
            isDashboardPage ? 'justify-center' : ''
          )}
          onDoubleClick={isMac ? handleToggleMaximize : undefined}
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
        {/* Windows: Decorum plugin provides native window controls, so we don't render our custom buttons */}
        {/* The decorum controls are injected into the DOM and styled via decorum.css */}
      </div>
    </div>
  )
}

export default TitleBar
