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
