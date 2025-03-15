import React, { useEffect, useState } from "react";
import Slider from "../ui/Slider";
import { useSetting } from "../../contexts/SettingContext";

const StorageSection: React.FC = () => {
  const { setting, loading, error, updateStorageSetting } = useSetting();

  // 本地状态
  const [historyRetentionDays, setHistoryRetentionDays] = useState(30);
  const [maxHistoryItems, setMaxHistoryItems] = useState(1000);

  // 最大历史记录数选项
  const maxHistoryOptions = [
    { value: 100, label: "100条" },
    { value: 500, label: "500条" },
    { value: 1000, label: "1000条" },
    { value: 5000, label: "5000条" },
    { value: 0, label: "无限制" },
  ];

  // 当设置加载完成后，更新本地状态
  useEffect(() => {
    if (setting) {
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
  const handleMaxHistoryItemsChange = (
    event: React.ChangeEvent<HTMLSelectElement>
  ) => {
    const value = parseInt(event.target.value, 10);
    setMaxHistoryItems(value);
    updateStorageSetting({ max_history_items: value });
  };

  // 处理清空历史记录
  const handleClearHistory = () => {
    // 这里可以添加清空历史记录的逻辑
    // 例如调用后端API来清空历史记录
    alert("历史记录已清空");
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
    <>
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
        <div className="flex items-center justify-between mb-2">
          <div>
            <h4 className="text-sm font-medium text-white">最大历史记录数</h4>
            <p className="text-xs text-gray-400 mt-0.5">
              限制保存的剪贴板历史记录数量
            </p>
          </div>
          <div className="w-36">
            <select
              className="w-full bg-gray-700 border border-gray-700 rounded-lg px-3 py-1.5 text-sm text-white focus:outline-none focus:ring-1 focus:ring-violet-400"
              value={maxHistoryItems}
              onChange={handleMaxHistoryItemsChange}
            >
              {maxHistoryOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>
        </div>
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
