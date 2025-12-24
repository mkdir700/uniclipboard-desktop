import DashboardPage from "@/pages/DashboardPage";
import DevicesPage from "@/pages/DevicesPage";
import SettingsPage from "@/pages/SettingsPage";
import OnboardingPage from "@/pages/OnboardingPage";
import {
  BrowserRouter as Router,
  Routes,
  Route,
  Navigate,
  useLocation,
  useNavigate,
  Outlet,
} from "react-router-dom";
import { SettingProvider } from "@/contexts/SettingContext";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import { MainLayout } from "@/layouts";
import "./App.css";

// 动画配置
const pageVariants = {
  initial: {
    opacity: 0,
    y: 20,
  },
  in: {
    opacity: 1,
    y: 0,
  },
  out: {
    opacity: 0,
    y: -20,
  },
};

const pageTransition = {
  type: "tween" as const,
  ease: "anticipate" as const,
  duration: 0.5,
};

// 引导页包装器组件，用于处理完成后的导航
const OnboardingWrapper = ({
  onOnboardingComplete,
}: {
  onOnboardingComplete: () => void;
}) => {
  const [isComplete, setIsComplete] = useState(false);

  const handleComplete = () => {
    // 调用父组件的回调函数更新 isOnboarded 状态
    console.log("引导完成，通知父组件");
    onOnboardingComplete();
    setIsComplete(true);
  };

  if (isComplete) {
    return (
      <div className="h-screen w-screen bg-gray-900 flex items-center justify-center">
        <motion.div
          initial={{ opacity: 0, scale: 0.8 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.5 }}
          className="flex flex-col items-center"
        >
          <div className="w-16 h-16 border-t-4 border-violet-500 border-solid rounded-full animate-spin mb-4"></div>
          <div className="text-violet-400 text-lg font-medium">
            准备您的仪表板...
          </div>
        </motion.div>
      </div>
    );
  }

  return <OnboardingPage onComplete={handleComplete} />;
};

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
  const [redirecting, setRedirecting] = useState(false);
  const [showDashboardAnimation, setShowDashboardAnimation] = useState(false);
  const navigate = useNavigate();
  const location = useLocation();

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

  // 路径变化时重置动画标志
  useEffect(() => {
    // 如果不是从引导页重定向过来，则不显示动画
    if (location.pathname === "/" && !redirecting) {
      setShowDashboardAnimation(false);
    }
  }, [location.pathname, redirecting]);

  // 添加函数来更新 isOnboarded 状态并导航
  const handleOnboardingComplete = () => {
    console.log("引导完成，更新状态为 true");
    setIsOnboarded(true);
    setRedirecting(true);
    setShowDashboardAnimation(true); // 设置显示仪表盘动画

    // 使用 setTimeout 给过渡动画一些时间
    setTimeout(() => {
      console.log("导航到主页");
      navigate("/");
      setRedirecting(false);
    }, 600);
  };

  if (loading) {
    return (
      <div className="h-screen w-screen bg-gray-900 flex items-center justify-center">
        <div className="animate-pulse text-violet-400">加载中...</div>
      </div>
    );
  }

  return (
    <SettingProvider>
      <Routes>
        <Route
          element={
            isOnboarded ? <AuthenticatedLayout /> : <Navigate to="/onboarding" />
          }
        >
          <Route
            path="/"
            element={
              showDashboardAnimation ? (
                <motion.div
                  initial="initial"
                  animate="in"
                  variants={pageVariants}
                  transition={pageTransition}
                  className="w-full h-full"
                  onAnimationComplete={() => {
                    // 动画完成后重置标志
                    setTimeout(() => setShowDashboardAnimation(false), 100);
                  }}
                >
                  <DashboardPage />
                </motion.div>
              ) : (
                <div className="w-full h-full">
                  <DashboardPage />
                </div>
              )
            }
          />
          <Route path="/devices" element={<DevicesPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Route>
        <Route
          path="/onboarding"
          element={
            isOnboarded && !redirecting ? (
              <Navigate to="/" />
            ) : (
              <OnboardingWrapper
                onOnboardingComplete={handleOnboardingComplete}
              />
            )
          }
        />
      </Routes>
    </SettingProvider>
  );
};

export default function App() {
  return (
    <Router>
      <AppContent />
    </Router>
  );
}
