import { listen } from '@tauri-apps/api/event'
import { useEffect, useState } from 'react'
import { BrowserRouter as Router, Routes, Route, Navigate, Outlet } from 'react-router-dom'
import { type EncryptionSessionStatus } from '@/api/security'
import { TitleBar } from '@/components'
import { Toaster } from '@/components/ui/sonner'
import { useOnboarding } from '@/contexts/onboarding-context'
import { OnboardingProvider } from '@/contexts/OnboardingContext'
import { useSearch } from '@/contexts/search-context'
import { SearchProvider } from '@/contexts/SearchContext'
import { SettingProvider } from '@/contexts/SettingContext'
import { ShortcutProvider } from '@/contexts/ShortcutContext'
import { UpdateProvider } from '@/contexts/UpdateContext'
import { usePlatform } from '@/hooks/usePlatform'
import { MainLayout, SettingsFullLayout, WindowShell } from '@/layouts'
import DashboardPage from '@/pages/DashboardPage'
import DevicesPage from '@/pages/DevicesPage'
import OnboardingPage from '@/pages/OnboardingPage'
import SettingsPage from '@/pages/SettingsPage'
import UnlockPage from '@/pages/UnlockPage'
import { useGetEncryptionSessionStatusQuery } from '@/store/api'
import './App.css'

// 认证布局包装器 - 保持 Sidebar 持久化
const AuthenticatedLayout = () => {
  return (
    <MainLayout>
      <Outlet />
    </MainLayout>
  )
}

// 主应用程序内容
const AppContent = () => {
  const { status, loading } = useOnboarding()
  const [encryptionStatus, setEncryptionStatus] = useState<EncryptionSessionStatus | null>(null)
  const [encryptionError, setEncryptionError] = useState<string | null>(null)
  const {
    data: encryptionData,
    isLoading: encryptionLoading,
    error: encryptionQueryError,
  } = useGetEncryptionSessionStatusQuery()

  useEffect(() => {
    const unlistenPromise = listen<'SessionReady' | { type: string }>(
      'encryption://event',
      event => {
        console.log('Encryption event:', event.payload)
        const eventType = typeof event.payload === 'string' ? event.payload : event.payload?.type
        if (eventType === 'SessionReady') {
          setEncryptionStatus(prev =>
            prev ? { ...prev, session_ready: true } : { initialized: true, session_ready: true }
          )
        }
      }
    )

    return () => {
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [])

  useEffect(() => {
    if (encryptionData) {
      setEncryptionStatus(encryptionData)
      setEncryptionError(null)
    }
  }, [encryptionData])

  useEffect(() => {
    if (!encryptionQueryError) {
      return
    }

    const message =
      typeof encryptionQueryError === 'object' && 'message' in encryptionQueryError
        ? String(encryptionQueryError.message)
        : 'Failed to check encryption status'
    setEncryptionError(message)
  }, [encryptionQueryError])

  const resolvedEncryptionStatus = encryptionStatus ?? encryptionData ?? null

  // Wait for onboarding status to load
  if (loading || status === null || encryptionLoading) {
    return null
  }

  if (encryptionError) {
    return (
      <div className="flex h-full w-full items-center justify-center p-4 text-sm text-foreground">
        <div className="max-w-sm rounded-md border border-border/20 bg-muted p-4 text-center">
          Failed to verify encryption status. Please restart the app.
        </div>
      </div>
    )
  }

  if (!status.has_completed) {
    return <OnboardingPage />
  }

  // If initialized but not ready, show unlock page
  if (resolvedEncryptionStatus?.initialized && !resolvedEncryptionStatus?.session_ready) {
    return <UnlockPage />
  }

  return (
    <ShortcutProvider>
      <Routes>
        <Route element={<AuthenticatedLayout />}>
          <Route
            path="/"
            element={
              <div className="w-full h-full">
                <DashboardPage />
              </div>
            }
          />
          <Route path="/devices" element={<DevicesPage />} />
        </Route>
        <Route element={<SettingsFullLayout />}>
          <Route path="/settings" element={<SettingsPage />} />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
      <Toaster />
    </ShortcutProvider>
  )
}

export default function App() {
  return (
    <Router>
      <SearchProvider>
        <OnboardingProvider>
          <SettingProvider>
            <UpdateProvider>
              <AppContentWithBar />
            </UpdateProvider>
          </SettingProvider>
        </OnboardingProvider>
      </SearchProvider>
    </Router>
  )
}

// TitleBar wrapper with search context
const TitleBarWithSearch = () => {
  const { searchValue, setSearchValue } = useSearch()
  return <TitleBar searchValue={searchValue} onSearchChange={setSearchValue} />
}

// App content with WindowShell structure
const AppContentWithBar = () => {
  // WindowShell provides the correct window-level structure:
  // - TitleBar: Window chrome layer (full-width, drag region)
  // - Content: App layout layer (Sidebar + Main via routes)
  const { isMac, isTauri } = usePlatform()
  const showCustomTitleBar = !isTauri || isMac

  return (
    <WindowShell titleBar={showCustomTitleBar ? <TitleBarWithSearch /> : null}>
      <AppContent />
    </WindowShell>
  )
}
