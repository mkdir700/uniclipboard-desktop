import React, { useEffect, useState } from "react";
import Slider from "../ui/Slider";
import { useSetting } from "../../contexts/SettingContext";
import { Select } from "../ui";

const StorageSection: React.FC = () => {
  const { setting, error, updateStorageSetting } = useSetting();

  // 本地状态
  const [historyRetentionDays, setHistoryRetentionDays] = useState(30);
  const [maxHistoryItems, setMaxHistoryItems] = useState(1000);
  const [autoClearHistory, setAutoClearHistory] = useState("never");

  // 自动清除选项
  const autoClearOptions = [
    { value: "never", label: "从不" },
    { value: "daily", label: "每天" },
    { value: "weekly", label: "每周" },
    { value: "monthly", label: "每月" },
    { value: "on_exit", label: "每次退出" },
  ];

  // 最大历史记录数选项
  const maxHistoryOptions = [
    { value: "100", label: "100条" },
    { value: "500", label: "500条" },
    { value: "1000", label: "1000条" },
    { value: "5000", label: "5000条" },
    { value: "0", label: "无限制" },
  ];

  // 当设置加载完成后，更新本地状态
  useEffect(() => {
    if (setting) {
      setAutoClearHistory(setting.storage.auto_clear_history);
      setHistoryRetentionDays(setting.storage.history_retention_days);
      setMaxHistoryItems(setting.storage.max_history_items);
    }
  }, [setting]);

  // 处理历史记录保留时间变化
  const handleHistoryRetentionChange = (value: number) => {
    setHistoryRetentionDays(value);
    updateStorageSetting({ history_retention_days: value });
  };

  // 处理最大历史记录数变化
  const handleMaxHistoryItemsChange = (value: string) => {
    const numValue = parseInt(value, 10);
    setMaxHistoryItems(numValue);
    updateStorageSetting({ max_history_items: numValue });
  };

  // 处理自动清除历史记录选项变化
  const handleAutoClearHistoryChange = (value: string) => {
    setAutoClearHistory(value);
    updateStorageSetting({ auto_clear_history: value });
  };

  // 处理清空历史记录
  const handleClearHistory = () => {
    // 这里可以添加清空历史记录的逻辑
    // 例如调用后端API来清空历史记录
    alert("历史记录已清空");
  };

  // 如果有错误，显示错误信息
  if (error) {
    return <div className="text-red-500 py-4">加载设置失败: {error}</div>;
  }

  return (
    <>
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

      {/* 存储使用情况 */}
      <div className="settings-item py-2 rounded-lg px-2">
        <div className="flex items-center justify-between mb-1">
          <h4 className="text-sm font-medium text-white">存储使用情况</h4>
          <span className="text-xs text-gray-400">128MB / 1GB</span>
        </div>
        <div className="w-full bg-gray-700 rounded-full h-2.5">
          <div
            className="bg-violet-500 h-2.5 rounded-full"
            style={{ width: "12.8%" }}
          ></div>
        </div>
        <div className="flex justify-between mt-1 text-xs text-gray-400">
          <span>已使用 12.8%</span>
        </div>
      </div>

      {/* 存储限制 */}
      <div className="settings-item py-2 rounded-lg px-2">
        <div className="w-full">
          <Slider
            min={1}
            max={90}
            value={historyRetentionDays}
            onChange={handleHistoryRetentionChange}
            label="历史记录保留时间"
            description="设置剪贴板历史记录的最长保留时间"
            unit="天"
            keyPoints={[
              { value: 7, label: "7D" },
              { value: 30, label: "30D" },
              { value: 60, label: "60D" },
              { value: 90, label: "90D" },
            ]}
          />
        </div>
      </div>

      {/* 最大历史记录数 */}
      <div className="settings-item py-2 rounded-lg px-2">
        <Select
          options={maxHistoryOptions}
          value={maxHistoryItems.toString()}
          onChange={handleMaxHistoryItemsChange}
          label="最大历史记录数"
          description="限制保存的剪贴板历史记录数量"
          width="w-36"
        />
      </div>

      {/* 清空历史记录 */}
      <div className="settings-item py-2 rounded-lg px-2">
        <div className="flex items-center justify-between">
          <div>
            <h4 className="text-sm font-medium text-white">历史记录</h4>
            <p className="text-xs text-gray-400 mt-0.5">
              清空所有剪贴板历史记录并释放存储空间
            </p>
          </div>
          <button
            className="px-3 py-1.5 bg-red-500/20 hover:bg-red-500/30 text-sm text-red-400 rounded-lg transition duration-150"
            onClick={handleClearHistory}
          >
            清空历史记录
          </button>
        </div>
      </div>
    </>
  );
};

export default StorageSection;
