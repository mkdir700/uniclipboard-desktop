import { BrowserRouter as Router, Routes, Route, Navigate, Outlet } from 'react-router-dom'
import { TitleBar } from '@/components'
import GlobalPairingRequestDialog from '@/components/GlobalPairingRequestDialog'
import PairingPinDialog from '@/components/PairingPinDialog'
import { Toaster } from '@/components/ui/sonner'
import { P2PProvider } from '@/contexts/P2PContext'
import { SearchProvider, useSearch } from '@/contexts/SearchContext'
import { SettingProvider } from '@/contexts/SettingContext'
import { ShortcutProvider } from '@/contexts/ShortcutContext'
import { useP2P } from '@/hooks/useP2P'
import { MainLayout, SettingsWindowLayout } from '@/layouts'
import DashboardPage from '@/pages/DashboardPage'
import DevicesPage from '@/pages/DevicesPage'
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
      {/* Global pairing dialogs */}
      <GlobalPairingDialogs />
    </MainLayout>
  )
}

// 主应用程序内容
const AppContent = () => {
  return (
    <ShortcutProvider>
      <P2PProvider>
        <SettingProvider>
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
            <Route element={<SettingsWindowLayout />}>
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
        <TitleBarWithSearch />
        <AppContent />
      </SearchProvider>
    </Router>
  )
}

// TitleBar wrapper with search context
const TitleBarWithSearch = () => {
  const { searchValue, setSearchValue } = useSearch()
  return <TitleBar searchValue={searchValue} onSearchChange={setSearchValue} />
}
