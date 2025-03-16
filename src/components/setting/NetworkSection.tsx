import React, { useEffect, useState } from "react";
import Select from "../ui/Select";
import { useSetting } from "../../contexts/SettingContext";

const NetworkSection: React.FC = () => {
  const { setting, error, updateNetworkSetting } = useSetting();

  // 本地状态
  const [syncMethod, setSyncMethod] = useState("lan_first");
  const [cloudServer, setCloudServer] = useState("api.clipsync.com");

  // 同步方式选项
  const syncMethodOptions = [
    { value: "lan_first", label: "优先使用局域网同步 (推荐)" },
    { value: "cloud_only", label: "仅使用云端同步" },
    { value: "lan_only", label: "仅使用局域网同步" },
  ];

  // 当设置加载完成后，更新本地状态
  useEffect(() => {
    if (setting) {
      setSyncMethod(setting.network.sync_method);
      setCloudServer(setting.network.cloud_server);
    }
  }, [setting]);

  // 处理同步方式变化
  const handleSyncMethodChange = (value: string) => {
    setSyncMethod(value);
    updateNetworkSetting({ sync_method: value });
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
      {/* 同步方式 */}
      <div className="settings-item py-2 rounded-lg px-2">
        <Select
          options={syncMethodOptions}
          value={syncMethod}
          onChange={handleSyncMethodChange}
          label="同步方式"
          description="选择同步方式"
          width="w-64"
        />
      </div>

      {/* 云服务器配置 */}
      <div className="settings-item py-2 rounded-lg px-2 opacity-60 cursor-not-allowed">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center">
            <h4 className="text-sm font-medium text-white">云服务器配置</h4>
            <span className="ml-2 px-1.5 py-0.5 bg-gray-700 text-xs text-gray-400 rounded">
              即将推出
            </span>
          </div>
          <button
            className="px-2 py-1 bg-gray-700 text-xs text-gray-300 rounded pointer-events-none"
            disabled
          >
            高级选项
          </button>
        </div>
        <div className="flex">
          <div className="px-2 py-1 bg-gray-700 rounded-lg text-sm text-gray-300 flex-1">
            使用默认云服务器 ({cloudServer})
          </div>
        </div>
      </div>
    </div>
  );
};

export default NetworkSection;
