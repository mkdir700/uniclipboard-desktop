import React from "react";
import { SettingContentLayout } from "../../layouts";

const StorageSection: React.FC = () => {
  return (
    <SettingContentLayout title="存储管理">
      {/* 存储使用情况 */}
      <div className="mb-4">
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
          <button className="text-violet-400 hover:text-violet-300">
            升级存储空间
          </button>
        </div>
      </div>

      <div className="space-y-4">
        {/* 存储限制 */}
        <div className="settings-item py-2 rounded-lg px-2">
          <div className="mb-2">
            <div className="flex items-center justify-between">
              <h4 className="text-sm font-medium text-white">
                历史记录保留时间
              </h4>
              <span className="text-xs text-violet-300">30天</span>
            </div>
            <p className="text-xs text-gray-400 mt-0.5">
              设置剪贴板历史记录的最长保留时间
            </p>
          </div>
          <div className="w-full">
            <input
              type="range"
              min="1"
              max="90"
              value="30"
              className="w-full h-2 bg-gray-600 rounded-lg appearance-none cursor-pointer accent-accent-500"
            />
          </div>
          <div className="flex justify-between mt-1 text-xs text-gray-400">
            <span>1天</span>
            <span>90天</span>
          </div>
        </div>

        {/* <!-- 最大历史记录数 --> */}
        <div className="settings-item py-2 rounded-lg px-2">
          <div className="flex items-center justify-between mb-2">
            <div>
              <h4 className="text-sm font-medium text-white">最大历史记录数</h4>
              <p className="text-xs text-gray-400 mt-0.5">
                限制保存的剪贴板历史记录数量
              </p>
            </div>
            <div className="w-36">
              <select className="w-full bg-gray-700 border border-gray-700 rounded-lg px-3 py-1.5 text-sm text-white focus:outline-none focus:ring-1 focus:ring-violet-400">
                <option>100条</option>
                <option>500条</option>
                <option selected>1000条</option>
                <option>5000条</option>
                <option>无限制</option>
              </select>
            </div>
          </div>
        </div>

        {/* <!-- 清理缓存 --> */}
        <div className="settings-item py-2 rounded-lg px-2">
          <div className="flex items-center justify-between">
            <div>
              <h4 className="text-sm font-medium text-white">缓存大小</h4>
              <p className="text-xs text-gray-400 mt-0.5">
                当前应用缓存占用空间
              </p>
            </div>
            <div className="flex items-center">
              <span className="text-sm text-gray-300 mr-3">45.2MB</span>
              <button className="px-3 py-1.5 bg-gray-700 hover:bg-gray-600 text-sm text-gray-300 rounded-lg transition duration-150">
                清理缓存
              </button>
            </div>
          </div>
        </div>

        {/* 清空历史记录 */}
        <div className="settings-item py-2 rounded-lg px-2">
          <div className="flex items-center justify-between">
            <div>
              <h4 className="text-sm font-medium text-white">历史记录</h4>
              <p className="text-xs text-gray-400 mt-0.5">
                清空所有剪贴板历史记录
              </p>
            </div>
            <button className="px-3 py-1.5 bg-red-500/20 hover:bg-red-500/30 text-sm text-red-400 rounded-lg transition duration-150">
              清空历史记录
            </button>
          </div>
        </div>
      </div>
    </SettingContentLayout>
  );
};

export default StorageSection;
