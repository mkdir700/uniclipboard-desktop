import React, { useEffect, useState } from "react";
import { Slider, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui";
import { useSetting } from "@/contexts/SettingContext";

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
  const handleHistoryRetentionChange = (value: number[]) => {
    const newValue = value[0];
    setHistoryRetentionDays(newValue);
    updateStorageSetting({ history_retention_days: newValue });
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
      <div className="py-2 rounded-lg px-2">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h4 className="text-base font-medium">自动清除历史记录</h4>
            <p className="text-sm text-muted-foreground">
              定期自动清除剪贴板历史记录
            </p>
          </div>
          <Select
            value={autoClearHistory}
            onValueChange={handleAutoClearHistoryChange}
          >
            <SelectTrigger className="w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {autoClearOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* 存储使用情况 */}
      <div className="py-2 rounded-lg px-2">
        <div className="flex items-center justify-between mb-2">
          <h4 className="text-base font-medium">存储使用情况</h4>
          <span className="text-sm text-muted-foreground">128MB / 1GB</span>
        </div>
        <div className="w-full bg-secondary rounded-full h-2.5">
          <div
            className="bg-primary h-2.5 rounded-full"
            style={{ width: "12.8%" }}
          ></div>
        </div>
        <div className="flex justify-between mt-1 text-xs text-muted-foreground">
          <span>已使用 12.8%</span>
        </div>
      </div>

      {/* 存储限制 */}
      <div className="py-2 rounded-lg px-2">
        <div className="w-full space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <h4 className="text-base font-medium">历史记录保留时间</h4>
              <p className="text-sm text-muted-foreground">
                设置剪贴板历史记录的最长保留时间
              </p>
            </div>
            <span className="text-sm text-muted-foreground">{historyRetentionDays} 天</span>
          </div>
          <Slider
            min={1}
            max={90}
            step={1}
            value={[historyRetentionDays]}
            onValueChange={handleHistoryRetentionChange}
            className="w-full"
          />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>7天</span>
            <span>30天</span>
            <span>60天</span>
            <span>90天</span>
          </div>
        </div>
      </div>

      {/* 最大历史记录数 */}
      <div className="py-2 rounded-lg px-2">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h4 className="text-base font-medium">最大历史记录数</h4>
            <p className="text-sm text-muted-foreground">
              限制保存的剪贴板历史记录数量
            </p>
          </div>
          <Select
            value={maxHistoryItems.toString()}
            onValueChange={handleMaxHistoryItemsChange}
          >
            <SelectTrigger className="w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {maxHistoryOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* 清空历史记录 */}
      <div className="py-2 rounded-lg px-2">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h4 className="text-base font-medium">历史记录</h4>
            <p className="text-sm text-muted-foreground">
              清空所有剪贴板历史记录并释放存储空间
            </p>
          </div>
          <button
            className="px-3 py-1.5 bg-destructive/10 hover:bg-destructive/20 text-sm text-destructive rounded-lg transition duration-150"
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
