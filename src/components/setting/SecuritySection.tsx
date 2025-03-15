import React, { useEffect, useState } from "react";
import Toggle from "../ui/Toggle";
import Select from "../ui/Select";
import { useSetting } from "../../contexts/SettingContext";

const SecuritySection: React.FC = () => {
  const { setting, loading, error, updateSecuritySetting } = useSetting();

  // 本地状态
  const [endToEndEncryption, setEndToEndEncryption] = useState(true);
  const [autoClearHistory, setAutoClearHistory] = useState("never");

  // 自动清除选项
  const autoClearOptions = [
    { value: "never", label: "从不" },
    { value: "daily", label: "每天" },
    { value: "weekly", label: "每周" },
    { value: "monthly", label: "每月" },
    { value: "on_exit", label: "每次退出" },
  ];

  // 当设置加载完成后，更新本地状态
  useEffect(() => {
    if (setting) {
      setEndToEndEncryption(setting.security.end_to_end_encryption);
      setAutoClearHistory(setting.security.auto_clear_history);
    }
  }, [setting]);

  // 处理端到端加密开关变化
  const handleEndToEndEncryptionChange = () => {
    const newValue = !endToEndEncryption;
    setEndToEndEncryption(newValue);
    updateSecuritySetting({ end_to_end_encryption: newValue });
  };

  // 处理自动清除历史记录选项变化
  const handleAutoClearHistoryChange = (value: string) => {
    setAutoClearHistory(value);
    updateSecuritySetting({ auto_clear_history: value });
  };

  // 如果正在加载，显示加载状态
  //   if (loading) {
  //     return <div className="text-center py-4">正在加载设置...</div>;
  //   }

  // 如果有错误，显示错误信息
  if (error) {
    return <div className="text-red-500 py-4">加载设置失败: {error}</div>;
  }

  return (
    <div className="space-y-4">
      {/* 端到端加密 */}
      <div className="settings-item py-2 rounded-lg px-2">
        <Toggle
          checked={endToEndEncryption}
          onChange={handleEndToEndEncryptionChange}
          label="端到端加密"
          description="启用后，所有同步内容将使用端到端加密传输"
        />
      </div>

      {/* 自动清除规则 */}
      <div className="settings-item py-2 rounded-lg px-2">
        <Select
          options={autoClearOptions}
          width="w-36"
          value={autoClearHistory}
          onChange={handleAutoClearHistoryChange}
          label="自动清除历史记录"
          description="定期自动清除剪贴板历史记录"
        />
      </div>
    </div>
  );
};

export default SecuritySection;
