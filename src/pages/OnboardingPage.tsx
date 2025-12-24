import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { Laptop, Copy, ArrowRight, Info, Edit } from "lucide-react";

interface OnboardingPageProps {
  onComplete?: () => void;
}

const OnboardingPage: React.FC<OnboardingPageProps> = ({ onComplete }) => {
  const [deviceId, setDeviceId] = useState<string>("");
  const [deviceAlias, setDeviceAlias] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [copied, setCopied] = useState<boolean>(false);
  const navigate = useNavigate();

  useEffect(() => {
    loadDeviceId();
  }, []);

  const loadDeviceId = async () => {
    try {
      setIsLoading(true);
      const id = await invoke("get_device_id");
      setDeviceId(id as string);
    } catch (error) {
      console.error("Failed to get device ID:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCopyId = () => {
    if (deviceId) {
      navigator.clipboard.writeText(deviceId);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const completeOnboarding = async () => {
    try {
      setIsLoading(true);
      await invoke("save_device_info", {
        alias: deviceAlias.trim() || "My Device",
      });
      await invoke("complete_onboarding");

      if (onComplete) {
        onComplete();
      } else {
        navigate("/");
      }
    } catch (error) {
      console.error("Failed to complete onboarding:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSkip = () => {
    completeOnboarding();
  };

  return (
    <div className="min-h-screen w-screen bg-slate-50 dark:bg-slate-950 flex flex-col items-center justify-center p-6 relative overflow-hidden transition-colors duration-200">
      {/* Background blur effects */}
      <div className="absolute top-0 left-0 w-full h-full overflow-hidden -z-10 pointer-events-none opacity-50">
        <div className="absolute top-[-10%] right-[-5%] w-96 h-96 bg-violet-500/10 rounded-full blur-3xl mix-blend-multiply dark:mix-blend-screen"></div>
        <div className="absolute bottom-[-10%] left-[-5%] w-96 h-96 bg-blue-500/10 rounded-full blur-3xl mix-blend-multiply dark:mix-blend-screen"></div>
      </div>

      <main className="w-full max-w-lg z-10 flex flex-col items-center">
        {/* Header */}
        <div className="flex flex-col items-center mb-10 text-center">
          <div className="w-14 h-14 bg-white dark:bg-slate-900 rounded-2xl shadow-sm border border-slate-200 dark:border-slate-800 flex items-center justify-center mb-6 text-violet-500">
            <Laptop className="w-6 h-6" />
          </div>
          <h1 className="text-3xl font-bold text-slate-900 dark:text-white tracking-tight mb-2">
            Device Setup
          </h1>
          <p className="text-slate-500 dark:text-slate-400 text-lg leading-relaxed max-w-sm">
            Connect this device to start syncing your clipboard history securely.
          </p>
        </div>

        <div className="w-full flex flex-col gap-8">
          {/* Device ID Section */}
          <div className="flex flex-col items-center">
            <div className="flex items-center gap-2 mb-3">
              <label className="text-xs font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                Current Device ID
              </label>
              <Info className="w-4 h-4 text-slate-400 cursor-help" />
            </div>

            <div className="w-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-2xl p-6 flex flex-col items-center justify-center relative group transition-all hover:border-violet-500/30">
              {/* Online status badge */}
              <div className="absolute top-3 right-3 flex items-center gap-1.5 bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400 px-2 py-1 rounded text-[10px] font-bold uppercase tracking-wide">
                <span className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse"></span>
                Online
              </div>

              {/* Device ID display */}
              <div className="flex items-center gap-3">
                <span className="font-mono text-3xl md:text-4xl text-slate-800 dark:text-slate-100 tracking-wider select-all font-medium">
                  {isLoading ? "..." : deviceId}
                </span>
              </div>

              {/* Copy button */}
              <button
                onClick={handleCopyId}
                disabled={!deviceId || isLoading}
                className="mt-4 flex items-center gap-2 text-sm font-medium text-slate-400 hover:text-violet-500 transition-colors py-1 px-3 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-800/50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <Copy className="w-[18px] h-[18px]" />
                {copied ? "Copied!" : "Copy ID"}
              </button>
            </div>
          </div>

          {/* Divider */}
          <div className="flex items-center gap-4 px-2 opacity-60">
            <div className="h-px bg-gradient-to-r from-transparent via-slate-300 dark:via-slate-600 to-transparent w-full"></div>
          </div>

          {/* Device Alias Section */}
          <div>
            <label className="block text-xs font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400 mb-1 ml-1" htmlFor="device-alias">
              Device Alias <span className="text-slate-400 font-normal lowercase opacity-70 ml-1">(optional)</span>
            </label>

            <div className="relative group">
              <input
                id="device-alias"
                type="text"
                value={deviceAlias}
                onChange={(e) => setDeviceAlias(e.target.value)}
                placeholder="e.g. Design MacBook Pro"
                className="block w-full px-1 py-4 bg-transparent text-slate-900 dark:text-slate-100 placeholder:text-slate-300 dark:placeholder:text-slate-600 border-0 border-b border-slate-200 dark:border-slate-700 focus:border-violet-500 focus:ring-0 transition-all text-xl font-medium"
                disabled={isLoading}
              />
              <div className="absolute bottom-0 left-0 w-0 h-0.5 bg-violet-500 transition-all duration-300 group-focus-within:w-full"></div>
              <span className="absolute right-0 top-1/2 -translate-y-1/2 text-slate-300 dark:text-slate-600 pointer-events-none group-focus-within:text-violet-500 transition-colors">
                <Edit className="w-5 h-5" />
              </span>
            </div>

            <p className="text-sm text-slate-500 dark:text-slate-400 mt-2 ml-1">
              Give this device a friendly name to identify clips later.
            </p>
          </div>

          {/* Action Buttons */}
          <div className="pt-4">
            <button
              onClick={completeOnboarding}
              disabled={isLoading}
              className="w-full py-4 bg-violet-500 hover:bg-violet-600 text-white rounded-xl font-semibold shadow-xl shadow-violet-500/20 hover:shadow-violet-500/30 active:scale-[0.98] transition-all flex items-center justify-center gap-2 group disabled:opacity-70 disabled:cursor-not-allowed disabled:active:scale-100"
            >
              <span className="text-lg">Get Started</span>
              <ArrowRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
            </button>
            <button
              onClick={handleSkip}
              disabled={isLoading}
              className="w-full mt-4 py-2 text-sm text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Skip setup for now
            </button>
          </div>
        </div>
      </main>

      {/* Footer */}
      <div className="absolute bottom-6 left-0 w-full text-center">
        <span className="text-[10px] uppercase tracking-widest text-slate-300 dark:text-slate-600 font-medium">
          UniClipboard v2.0
        </span>
      </div>
    </div>
  );
};

export default OnboardingPage;
