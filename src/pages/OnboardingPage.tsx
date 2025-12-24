import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { AnimatePresence, motion } from "framer-motion";
import { Input } from "@/components/ui";
import { Clipboard } from "lucide-react";

// 引导步骤枚举
enum OnboardingStep {
  Welcome = 0,
  DeviceSetup = 1,
}

// 添加onComplete属性
interface OnboardingPageProps {
  onComplete?: () => void;
}

const OnboardingPage: React.FC<OnboardingPageProps> = ({ onComplete }) => {
  const [currentStep, setCurrentStep] = useState<OnboardingStep>(
    OnboardingStep.Welcome
  );
  const [deviceId, setDeviceId] = useState<string>("");
  const [deviceName, setDeviceName] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [direction, setDirection] = useState<number>(1); // 1 表示前进，-1 表示后退
  const navigate = useNavigate();

  // 生成设备ID
  useEffect(() => {
    if (currentStep === OnboardingStep.DeviceSetup && !deviceId) {
      loadDeviceId();
    }
  }, [currentStep]);

  const loadDeviceId = async () => {
    try {
      setIsLoading(true);
      const deviceId = await invoke("get_device_id");
      setDeviceId(deviceId as string);
    } catch (error) {
      console.error("获取设备ID失败:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleNext = () => {
    if (currentStep < OnboardingStep.DeviceSetup) {
      setDirection(1);
      setCurrentStep(currentStep + 1);
    } else {
      completeOnboarding();
    }
  };

  const handleBack = () => {
    if (currentStep > OnboardingStep.Welcome) {
      setDirection(-1);
      setCurrentStep(currentStep - 1);
    }
  };

  const completeOnboarding = async () => {
    try {
      setIsLoading(true);
      // 保存设备信息
      console.log("保存设备信息...");
      await invoke("save_device_info", {
        alias: deviceName.trim() || `我的设备`,
      });
      // 标记引导完成
      console.log("标记引导完成...");
      await invoke("complete_onboarding");
      console.log("引导完成标记成功");

      // 如果提供了onComplete回调函数，则调用它
      if (onComplete) {
        console.log("调用onComplete回调函数");
        onComplete();
      } else {
        console.log("无onComplete回调，直接导航到主页");
        // 使用navigate代替window.location.href进行导航
        navigate("/");
      }
    } catch (error) {
      console.error("完成引导设置失败:", error);
    } finally {
      setIsLoading(false);
    }
  };

  // 页面过渡变体
  const variants = {
    enter: (direction: number) => ({
      x: direction > 0 ? 250 : -250,
      opacity: 0,
    }),
    center: {
      x: 0,
      opacity: 1,
    },
    exit: (direction: number) => ({
      x: direction < 0 ? 250 : -250,
      opacity: 0,
    }),
  };

  // 欢迎页面
  const renderWelcomePage = () => (
    <motion.div
      key="welcome"
      custom={direction}
      variants={variants}
      initial="enter"
      animate="center"
      exit="exit"
      transition={{ type: "spring", stiffness: 300, damping: 30 }}
      className="flex flex-col items-center justify-center h-full"
    >
      <motion.div
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ delay: 0.2, duration: 0.5 }}
        className="w-24 h-24 mb-6 rounded-2xl bg-gradient-to-br from-primary to-primary/80 flex items-center justify-center"
      >
        <Clipboard className="h-12 w-12 text-white" />
      </motion.div>
      <motion.h1
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ delay: 0.3, duration: 0.5 }}
        className="text-3xl font-bold text-white mb-4"
      >
        UniClipboard
      </motion.h1>
      <motion.p
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ delay: 0.4, duration: 0.5 }}
        className="text-gray-300 text-center max-w-md mb-8"
      >
        一款开源的跨设备剪贴板同步工具
      </motion.p>

      <motion.button
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ delay: 0.5, duration: 0.5 }}
        whileHover={{ scale: 1.05 }}
        whileTap={{ scale: 0.95 }}
        onClick={handleNext}
        className="px-6 py-2.5 bg-violet-500 hover:bg-violet-600 text-white rounded-lg transition duration-200 shadow-lg shadow-violet-500/20"
      >
        开始设置
      </motion.button>
    </motion.div>
  );

  // 设备设置页面
  const renderDeviceSetupPage = () => (
    <motion.div
      key="device-setup"
      custom={direction}
      variants={variants}
      initial="enter"
      animate="center"
      exit="exit"
      transition={{ type: "spring", stiffness: 300, damping: 30 }}
      className="flex flex-col items-center justify-center h-full"
    >
      <motion.h2
        initial={{ y: -20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ delay: 0.1, duration: 0.4 }}
        className="text-2xl font-bold text-white mb-6"
      >
        设置您的设备
      </motion.h2>

      <motion.div
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ delay: 0.2, duration: 0.4 }}
        className="bg-gray-800 p-6 rounded-lg mb-6 max-w-md w-full shadow-xl"
      >
        <div className="mb-4">
          <h4 className="text-sm font-medium text-white mb-1">设备 ID</h4>
          <div className="bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-sm text-white">
            {isLoading ? "生成中..." : deviceId}
          </div>
          <p className="text-xs text-gray-400 mt-1">
            设备 ID 是唯一的标识符，生成后无法更改
          </p>
        </div>

        <div className="mb-4">
          <h4 className="text-sm font-medium text-white mb-1">设备名称</h4>
          <Input
            value={deviceName}
            onChange={(e) => setDeviceName(e.target.value)}
            placeholder="我的设备"
            className="bg-muted border-border hover:border-primary focus:border-primary focus:ring-primary"
          />
          <p className="text-xs text-muted-foreground mt-1">
            为您的设备设置一个易于识别的名称
          </p>
        </div>
      </motion.div>

      <div className="flex space-x-4">
        <motion.button
          initial={{ x: -20, opacity: 0 }}
          animate={{ x: 0, opacity: 1 }}
          transition={{ delay: 0.3, duration: 0.4 }}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.95 }}
          onClick={handleBack}
          className="px-6 py-2.5 bg-gray-700 hover:bg-gray-600 text-white rounded-lg transition duration-200"
        >
          返回
        </motion.button>
        <motion.button
          initial={{ x: 20, opacity: 0 }}
          animate={{ x: 0, opacity: 1 }}
          transition={{ delay: 0.4, duration: 0.4 }}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.95 }}
          onClick={handleNext}
          disabled={isLoading}
          className={`px-6 py-2.5 bg-violet-500 hover:bg-violet-600 text-white rounded-lg transition duration-200 shadow-lg shadow-violet-500/20 ${
            isLoading ? "opacity-70 cursor-not-allowed" : ""
          }`}
        >
          {isLoading ? "处理中..." : "完成设置"}
        </motion.button>
      </div>
    </motion.div>
  );

  return (
    <div className="h-screen w-screen bg-gray-900 flex items-center justify-center p-4">
      <div className="w-full max-w-2xl">
        {/* 步骤指示器 */}
        <div className="mb-10">
          <div className="flex justify-center mb-4">
            <div className="w-full max-w-md flex items-center">
              {[OnboardingStep.Welcome, OnboardingStep.DeviceSetup].map(
                (step, index) => (
                  <React.Fragment key={step}>
                    {/* 步骤圆点 */}
                    <div className="relative">
                      <motion.div
                        initial={false}
                        animate={{
                          scale: currentStep >= step ? 1 : 0.8,
                          backgroundColor:
                            currentStep >= step
                              ? "rgb(139, 92, 246)"
                              : "rgb(75, 85, 99)",
                        }}
                        className={`flex items-center justify-center w-9 h-9 rounded-full text-white font-medium text-sm z-10 relative transition-colors`}
                      >
                        {currentStep > step ? (
                          <motion.svg
                            initial={{ opacity: 0, scale: 0 }}
                            animate={{ opacity: 1, scale: 1 }}
                            transition={{ duration: 0.3 }}
                            xmlns="http://www.w3.org/2000/svg"
                            className="h-5 w-5"
                            viewBox="0 0 20 20"
                            fill="currentColor"
                          >
                            <path
                              fillRule="evenodd"
                              d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                              clipRule="evenodd"
                            />
                          </motion.svg>
                        ) : (
                          index + 1
                        )}
                      </motion.div>
                    </div>

                    {/* 连接线 */}
                    {index <
                      [OnboardingStep.Welcome, OnboardingStep.DeviceSetup]
                        .length -
                        1 && (
                      <motion.div
                        initial={{ scaleX: 0 }}
                        animate={{
                          scaleX: 1,
                          backgroundColor:
                            currentStep > step
                              ? "rgb(139, 92, 246)"
                              : "rgb(75, 85, 99)",
                        }}
                        transition={{ duration: 0.5 }}
                        className="h-1 flex-1 origin-left"
                      />
                    )}
                  </React.Fragment>
                )
              )}
            </div>
          </div>
        </div>

        {/* 内容区域 */}
        <div className="relative overflow-hidden" style={{ height: "400px" }}>
          <AnimatePresence initial={false} custom={direction} mode="wait">
            {currentStep === OnboardingStep.Welcome && renderWelcomePage()}
            {currentStep === OnboardingStep.DeviceSetup &&
              renderDeviceSetupPage()}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
};

export default OnboardingPage;
