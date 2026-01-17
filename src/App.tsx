import { listen } from '@tauri-apps/api/event'
import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { BrowserRouter as Router, Routes, Route, Navigate, Outlet } from 'react-router-dom'
import { TitleBar } from '@/components'
import GlobalPairingRequestDialog from '@/components/GlobalPairingRequestDialog'
import { LoadingScreen } from '@/components/LoadingScreen'
import PairingPinDialog from '@/components/PairingPinDialog'
import { Toaster } from '@/components/ui/sonner'
import { OnboardingProvider, useOnboarding } from '@/contexts/OnboardingContext'
import { P2PProvider } from '@/contexts/P2PContext'
import { SearchProvider, useSearch } from '@/contexts/SearchContext'
import { SettingProvider } from '@/contexts/SettingContext'
import { ShortcutProvider } from '@/contexts/ShortcutContext'
import { useP2P } from '@/hooks/useP2P'
import { MainLayout, SettingsFullLayout } from '@/layouts'
import DashboardPage from '@/pages/DashboardPage'
import DevicesPage from '@/pages/DevicesPage'
import OnboardingPage from '@/pages/OnboardingPage'
import SettingsPage from '@/pages/SettingsPage'
import './App.css'

// Global pairing dialogs component
const GlobalPairingDialogs = () => {
  const p2p = useP2P()

  return (
    <>
      {/* Global pairing request dialog */}
      <GlobalPairingRequestDialog
        open={p2p.showRequestDialog}
        request={p2p.pendingRequest}
        onAccept={p2p.acceptRequest}
        onReject={p2p.rejectRequest}
      />

      {/* PIN verification dialog */}
      <PairingPinDialog
        open={p2p.showPinDialog}
        onClose={p2p.closePinDialog}
        pinCode={p2p.pinData?.pin || ''}
        peerDeviceName={p2p.pinData?.peerDeviceName}
        isInitiator={false}
        onConfirm={p2p.verifyPin}
      />
    </>
  )
}

// 认证布局包装器 - 保持 Sidebar 持久化
const AuthenticatedLayout = () => {
  return (
    <MainLayout>
      <Outlet />
    </MainLayout>
  )
}

// Global overlays that must be rendered regardless of route/layout
const GlobalOverlays = () => {
  return <GlobalPairingDialogs />
}

// 主应用程序内容
const AppContent = () => {
  const { status, loading } = useOnboarding()
  const { t } = useTranslation()

  // Backend loading state
  const [backendReady, setBackendReady] = useState(false)
  const [fadingOut, setFadingOut] = useState(false)
  const [initError, setInitError] = useState<string | null>(null)

  // Listen for backend-ready event
  useEffect(() => {
    // Timeout protection (30 seconds)
    const timeoutId = setTimeout(() => {
      if (!backendReady && !fadingOut) {
        setInitError(t('loading.timeout_error'))
      }
    }, 30000)

    // Listen for backend-ready event
    const unlistenPromise = listen('backend-ready', () => {
      clearTimeout(timeoutId)

      // Trigger fade-out animation first
      setFadingOut(true)

      // Switch to main app after fade-out completes
      setTimeout(() => {
        setBackendReady(true)
      }, 300)
    })

    return () => {
      clearTimeout(timeoutId)
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [t])

  // Show error screen if initialization failed
  if (initError) {
    return (
      <div className="h-screen w-screen flex items-center justify-center bg-background">
        <div className="text-center">
          <div className="text-destructive mb-4">{t('loading.error_title')}</div>
          <div className="text-muted-foreground text-sm">{initError}</div>
        </div>
      </div>
    )
  }

  // Show loading screen if backend not ready
  if (!backendReady) {
    return (
      <LoadingScreen className={fadingOut ? 'opacity-0 transition-opacity duration-300' : ''} />
    )
  }

  if (loading || status === null) {
    return null // Loading
  }

  if (!status.has_completed) {
    return <OnboardingPage />
  }

  return (
    <ShortcutProvider>
      <P2PProvider>
        <SettingProvider>
          <GlobalOverlays />
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
        </SettingProvider>
      </P2PProvider>
    </ShortcutProvider>
  )
}

export default function App() {
  return (
    <Router>
      <SearchProvider>
        <OnboardingProvider>
          <TitleBarWithSearch />
          <AppContent />
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
