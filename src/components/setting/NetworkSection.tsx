import React from "react";

const NetworkSection: React.FC = () => {
  return (
    <div className="space-y-4">
      {/* <!-- 同步方式 --> */}
      <div className="settings-item py-2 rounded-lg px-2">
        <h4 className="text-sm font-medium text-white mb-2">同步方式</h4>
        <div className="space-y-2">
          <label className="flex items-center">
            <input
              type="radio"
              name="sync-method"
              className="form-radio h-4 w-4 text-violet-500 focus:ring-0 focus:ring-offset-0 border-gray-600 bg-gray-700"
              checked
            />
            <span className="ml-2 text-sm text-gray-300">
              优先使用局域网同步 (推荐)
            </span>
          </label>
          <label className="flex items-center">
            <input
              type="radio"
              name="sync-method"
              className="form-radio h-4 w-4 text-violet-500 focus:ring-0 focus:ring-offset-0 border-gray-600 bg-gray-700"
            />
            <span className="ml-2 text-sm text-gray-300">仅使用云端同步</span>
          </label>
          <label className="flex items-center">
            <input
              type="radio"
              name="sync-method"
              className="form-radio h-4 w-4 text-violet-500 focus:ring-0 focus:ring-offset-0 border-gray-600 bg-gray-700"
            />
            <span className="ml-2 text-sm text-gray-300">仅使用局域网同步</span>
          </label>
        </div>
      </div>

      {/* 仅在WIFI下同步 */}
      <div className="flex items-center justify-between settings-item py-2 rounded-lg px-2">
        <div>
          <h4 className="text-sm font-medium text-white">仅在WIFI下同步</h4>
          <p className="text-xs text-gray-400 mt-0.5">
            在移动设备上仅使用WIFI网络同步内容
          </p>
        </div>
        <label className="flex items-center cursor-pointer">
          <div className="relative">
            <input type="checkbox" className="sr-only" checked />
            <div className="toggle-bg w-11 h-6 bg-gray-600 rounded-full"></div>
          </div>
        </label>
      </div>

      {/* 后台同步 */}
      <div className="flex items-center justify-between settings-item py-2 rounded-lg px-2">
        <div>
          <h4 className="text-sm font-medium text-white">后台同步</h4>
          <p className="text-xs text-gray-400 mt-0.5">
            即使应用未运行也保持同步(可能增加电池消耗)
          </p>
        </div>
        <label className="flex items-center cursor-pointer">
          <div className="relative">
            <input type="checkbox" className="sr-only" checked />
            <div className="toggle-bg w-11 h-6 bg-gray-600 rounded-full"></div>
          </div>
        </label>
      </div>

      {/* 云服务器配置 */}
      <div className="settings-item py-2 rounded-lg px-2">
        <div className="flex items-center justify-between mb-2">
          <h4 className="text-sm font-medium text-white">云服务器配置</h4>
          <button className="px-2 py-1 bg-gray-700 hover:bg-gray-600 text-xs text-gray-300 rounded">
            高级选项
          </button>
        </div>
        <div className="flex">
          <div className="px-2 py-1 bg-gray-700 rounded-lg text-sm text-gray-300 flex-1">
            使用默认云服务器 (api.clipsync.com)
          </div>
        </div>
      </div>
    </div>
  );
};

export default NetworkSection;
