import DashboardPage from "@/pages/DashboardPage";
import DevicesPage from "@/pages/DevicesPage";
import SettingsPage from "@/pages/SettingsPage";
import {
  BrowserRouter as Router,
  Routes,
  Route,
  Navigate,
  Outlet,
} from "react-router-dom";
import { SettingProvider } from "@/contexts/SettingContext";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MainLayout } from "@/layouts";
import "./App.css";

// 认证布局包装器 - 保持 Sidebar 持久化
const AuthenticatedLayout = () => {
  return (
    <MainLayout>
      <Outlet />
    </MainLayout>
  );
};

// 主应用程序内容
const AppContent = () => {
  const [isOnboarded, setIsOnboarded] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const checkOnboardingStatus = async () => {
      try {
        const status = await invoke("check_onboarding_status");
        console.log("引导状态检查结果:", status);
        setIsOnboarded(!!status);
      } catch (error) {
        console.error("检查引导状态失败:", error);
        setIsOnboarded(false);
      } finally {
        setLoading(false);
      }
    };

    checkOnboardingStatus();
  }, []);

  // 自动完成引导（如果未完成）
  useEffect(() => {
    if (isOnboarded === false) {
      invoke("complete_onboarding").catch((error) => {
        console.error("自动完成引导失败:", error);
      });
    }
  }, [isOnboarded]);

  if (loading || isOnboarded === false) {
    return (
      <div className="h-screen w-screen bg-gray-900 flex items-center justify-center">
        <div className="animate-pulse text-violet-400">加载中...</div>
      </div>
    );
  }

  return (
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
          <Route path="/settings" element={<SettingsPage />} />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </SettingProvider>
  );
};

import { TitleBar } from "@/components";

export default function App() {
  return (
    <Router>
      <TitleBar />
      <AppContent />
    </Router>
  );
}
