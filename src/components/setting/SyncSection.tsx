import React from "react";

const SyncSection: React.FC = () => {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between settings-item py-2 rounded-lg px-2">
        <div>
          <h4 className="text-sm font-medium text-white">自动同步</h4>
          <p className="text-xs text-gray-400 mt-0.5">
            启用后，ClipSync将自动同步您复制的内容到所有设备
          </p>
        </div>
        <label className="flex items-center cursor-pointer">
          <div className="relative">
            <input type="checkbox" className="sr-only" checked />
            <div className="toggle-bg w-11 h-6 bg-gray-600 rounded-full"></div>
          </div>
        </label>
      </div>

      <div className="settings-item py-2 rounded-lg px-2">
        <div className="flex items-center justify-between mb-2">
          <div>
            <h4 className="text-sm font-medium text-white">同步频率</h4>
            <p className="text-xs text-gray-400 mt-0.5">
              控制ClipSync检查新内容的频率
            </p>
          </div>
          <div className="w-36">
            <select className="w-full bg-gray-700 border border-gray-700 rounded-lg px-3 py-1.5 text-sm text-white focus:outline-none focus:ring-1 focus:ring-violet-400">
              <option>实时同步</option>
              <option>每30秒</option>
              <option>每分钟</option>
              <option>每5分钟</option>
              <option>每15分钟</option>
            </select>
          </div>
        </div>
      </div>

      <div className="settings-item py-2 rounded-lg px-2">
        <h4 className="text-sm font-medium text-white mb-2">同步内容类型</h4>
        <div className="grid grid-cols-2 gap-3">
          <label className="flex items-center">
            <input
              type="checkbox"
              className="form-checkbox h-4 w-4 text-violet-500 rounded focus:ring-0 focus:ring-offset-0 border-gray-600 bg-gray-700"
              checked
            />
            <span className="ml-2 text-sm text-gray-300">文本</span>
          </label>
          <label className="flex items-center">
            <input
              type="checkbox"
              className="form-checkbox h-4 w-4 text-violet-500 rounded focus:ring-0 focus:ring-offset-0 border-gray-600 bg-gray-700"
              checked
            />
            <span className="ml-2 text-sm text-gray-300">图片</span>
          </label>
          <label className="flex items-center">
            <input
              type="checkbox"
              className="form-checkbox h-4 w-4 text-violet-500 rounded focus:ring-0 focus:ring-offset-0 border-gray-600 bg-gray-700"
              checked
            />
            <span className="ml-2 text-sm text-gray-300">链接</span>
          </label>
          <label className="flex items-center">
            <input
              type="checkbox"
              className="form-checkbox h-4 w-4 text-violet-500 rounded focus:ring-0 focus:ring-offset-0 border-gray-600 bg-gray-700"
              checked
            />
            <span className="ml-2 text-sm text-gray-300">文件</span>
          </label>
          <label className="flex items-center">
            <input
              type="checkbox"
              className="form-checkbox h-4 w-4 text-violet-500 rounded focus:ring-0 focus:ring-offset-0 border-gray-600 bg-gray-700"
              checked
            />
            <span className="ml-2 text-sm text-gray-300">代码片段</span>
          </label>
          <label className="flex items-center">
            <input
              type="checkbox"
              className="form-checkbox h-4 w-4 text-violet-500 rounded focus:ring-0 focus:ring-offset-0 border-gray-600 bg-gray-700"
              checked
            />
            <span className="ml-2 text-sm text-gray-300">富文本</span>
          </label>
        </div>
      </div>

      <div className="settings-item py-2 rounded-lg px-2">
        <div className="mb-2">
          <div className="flex items-center justify-between">
            <h4 className="text-sm font-medium text-white">最大同步文件大小</h4>
            <span className="text-xs text-violet-300">10MB</span>
          </div>
          <p className="text-xs text-gray-400 mt-0.5">
            限制单个文件的最大同步大小
          </p>
        </div>
        <div className="w-full">
          <input
            type="range"
            min="1"
            max="50"
            value="10"
            className="w-full h-2 bg-gray-600 rounded-lg appearance-none cursor-pointer accent-accent-500"
          />
        </div>
      </div>
    </div>
  );
};

export default SyncSection;
