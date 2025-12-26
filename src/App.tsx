import { BrowserRouter as Router, Routes, Route, Navigate, Outlet } from 'react-router-dom'
import { TitleBar } from '@/components'
import { Toaster } from '@/components/ui/sonner'
import { SettingProvider } from '@/contexts/SettingContext'
import { ShortcutProvider } from '@/contexts/ShortcutContext'
import { MainLayout } from '@/layouts'
import DashboardPage from '@/pages/DashboardPage'
import DevicesPage from '@/pages/DevicesPage'
import SettingsPage from '@/pages/SettingsPage'
import './App.css'

// 认证布局包装器 - 保持 Sidebar 持久化
const AuthenticatedLayout = () => {
  return (
    <MainLayout>
      <Outlet />
    </MainLayout>
  )
}

// Settings 页面布局 - 不包含主 Sidebar
const SettingsLayout = () => {
  return (
    <div className="h-screen w-full flex bg-background text-foreground transition-colors duration-200">
      <Outlet />
    </div>
  )
}

// 主应用程序内容
const AppContent = () => {
  return (
    <ShortcutProvider>
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
          <Route element={<SettingsLayout />}>
            <Route path="/settings" element={<SettingsPage />} />
          </Route>
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
        <Toaster />
      </SettingProvider>
    </ShortcutProvider>
  )
}

export default function App() {
  return (
    <Router>
      <TitleBar />
      <AppContent />
    </Router>
  )
}
