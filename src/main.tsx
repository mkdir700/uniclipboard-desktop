import { attachConsole } from '@tauri-apps/plugin-log'
import React from 'react'
import ReactDOM from 'react-dom/client'
import { Provider } from 'react-redux'
import App from './App'
import './i18n'
import { store } from './store'
import { initSentry, Sentry } from '@/observability/sentry'

initSentry()

const startupTimingOrigin = Date.now()
const logStartupTiming = (label: string) => {
  const elapsed = Date.now() - startupTimingOrigin
  console.log(`[StartupTiming] ${label} t=${elapsed}ms`)
}

logStartupTiming('main.tsx module init')

if (typeof window !== 'undefined') {
  window.addEventListener('DOMContentLoaded', () => {
    logStartupTiming('DOMContentLoaded')
  })
  window.addEventListener('load', () => {
    logStartupTiming('window load')
  })
}

const notifyBackendFrontendReady = async () => {
  try {
    if (typeof window === 'undefined') {
      return
    }

    const importStartedAt = Date.now()
    console.log(
      `[StartupTiming] frontend_ready import start ts=${new Date(importStartedAt).toISOString()}`
    )
    const { invokeWithTrace } = await import('@/lib/tauri-command')
    console.log(
      `[StartupTiming] frontend_ready import end duration=${Date.now() - importStartedAt}ms`
    )
    const startAt = Date.now()
    console.log(`[StartupTiming] frontend_ready start ts=${new Date(startAt).toISOString()}`)
    console.log('[Startup] Attempting frontend_ready handshake')
    const deadline = Date.now() + 15000
    let lastError: unknown = null

    while (Date.now() < deadline) {
      try {
        await invokeWithTrace('frontend_ready')
        console.log('[Startup] frontend_ready sent')
        console.log(`[StartupTiming] frontend_ready success duration=${Date.now() - startAt}ms`)
        return
      } catch (error) {
        lastError = error
        await new Promise<void>(resolve => setTimeout(resolve, 100))
      }
    }

    console.warn('[Startup] frontend_ready handshake timed out:', lastError)
    console.warn(`[StartupTiming] frontend_ready timeout duration=${Date.now() - startAt}ms`)
  } catch (error) {
    console.error('[Startup] Failed to notify backend frontend_ready:', error)
    console.error('[StartupTiming] frontend_ready failed')
  }
}

const applyPlatformTypographyScale = () => {
  if (typeof navigator === 'undefined' || typeof document === 'undefined') {
    return
  }

  const ua = navigator.userAgent || ''
  const isWindows = ua.includes('Windows')

  if (!isWindows) {
    return
  }

  const root = document.documentElement

  root.style.setProperty('--font-size-sm', '0.75rem')
  root.style.setProperty('--font-size-base', '0.875rem')
  root.style.setProperty('--font-size-lg', '1rem')
  root.style.setProperty('--line-height-ui', '1.35')
}

applyPlatformTypographyScale()

// 初始化日志系统：将后端日志输出到浏览器 DevTools
const initLogging = async () => {
  try {
    // 仅在 Tauri 环境中运行（不在浏览器开发模式中）
    if (typeof window !== 'undefined' && '__TAURI__' in window) {
      await attachConsole()
      console.log('[Tauri Log] Console attached successfully')
    }
  } catch (error) {
    console.error('[Tauri Log] Failed to attach console:', error)
  }
}

// 执行日志初始化
initLogging().then(() => {
  console.log('[Tauri Log] Logging system initialized')
})

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <Provider store={store}>
      <Sentry.ErrorBoundary fallback={<div>Something went wrong.</div>}>
        <App />
      </Sentry.ErrorBoundary>
    </Provider>
  </React.StrictMode>
)

logStartupTiming('ReactDOM.render invoked')

// Notify backend after initial mount scheduling.
// React.StrictMode may mount twice in dev; backend side is idempotent.
queueMicrotask(() => {
  logStartupTiming('frontend_ready microtask executing')
  void notifyBackendFrontendReady()
})
