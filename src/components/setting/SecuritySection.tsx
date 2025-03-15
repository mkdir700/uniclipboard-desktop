import React from "react";

const SecuritySection: React.FC = () => {
  return (
    <div className="space-y-4">
      {/* 加密选项 */}
      <div className="flex items-center justify-between settings-item py-2 rounded-lg px-2">
        <div>
          <h4 className="text-sm font-medium text-white">端到端加密</h4>
          <p className="text-xs text-gray-400 mt-0.5">
            启用后，所有同步内容将使用端到端加密传输
          </p>
        </div>
        <label className="flex items-center cursor-pointer">
          <div className="relative">
            <input type="checkbox" className="sr-only" checked />
            <div className="toggle-bg w-11 h-6 bg-gray-600 rounded-full"></div>
          </div>
        </label>
      </div>

      {/* 密码保护 */}
      <div className="flex items-center justify-between settings-item py-2 rounded-lg px-2">
        <div>
          <h4 className="text-sm font-medium text-white">应用密码保护</h4>
          <p className="text-xs text-gray-400 mt-0.5">
            使用密码或生物识别保护访问ClipSync应用
          </p>
        </div>
        <label className="flex items-center cursor-pointer">
          <div className="relative">
            <input type="checkbox" className="sr-only" />
            <div className="toggle-bg w-11 h-6 bg-gray-600 rounded-full"></div>
          </div>
        </label>
      </div>

      {/* 敏感内容过滤 */}
      <div className="flex items-center justify-between settings-item py-2 rounded-lg px-2">
        <div>
          <h4 className="text-sm font-medium text-white">敏感内容过滤</h4>
          <p className="text-xs text-gray-400 mt-0.5">
            自动检测并阻止同步可能包含敏感信息的内容
          </p>
        </div>
        <label className="flex items-center cursor-pointer">
          <div className="relative">
            <input type="checkbox" className="sr-only" checked />
            <div className="toggle-bg w-11 h-6 bg-gray-600 rounded-full"></div>
          </div>
        </label>
      </div>

      {/* <!-- 敏感关键词设置 --> */}
      <div className="settings-item py-2 rounded-lg px-2">
        <h4 className="text-sm font-medium text-white mb-2">
          敏感关键词
        </h4>
        <p className="text-xs text-gray-400 mb-2">
          包含以下关键词的内容将不会被同步
        </p>
        <div className="flex flex-wrap gap-2 mb-2">
          <div className="bg-gray-700 rounded-full px-3 py-1 text-xs text-gray-300 flex items-center">
            密码
            <button className="ml-1.5 text-gray-400 hover:text-white">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-3.5 w-3.5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
          <div className="bg-gray-700 rounded-full px-3 py-1 text-xs text-gray-300 flex items-center">
            信用卡
            <button className="ml-1.5 text-gray-400 hover:text-white">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-3.5 w-3.5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
          <div className="bg-gray-700 rounded-full px-3 py-1 text-xs text-gray-300 flex items-center">
            身份证
            <button className="ml-1.5 text-gray-400 hover:text-white">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-3.5 w-3.5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
        </div>
        <div className="flex items-center">
          <input
            type="text"
            className="flex-1 bg-gray-700 border border-gray-700 rounded-l-lg px-3 py-1.5 text-sm text-white focus:outline-none focus:ring-1 focus:ring-violet-400"
            placeholder="添加新关键词..."
          />
          <button className="bg-violet-500 hover:bg-violet-400 text-white px-3 py-1.5 rounded-r-lg text-sm transition duration-150">
            添加
          </button>
        </div>
      </div>

      {/* 自动清除规则 */}
      <div className="settings-item py-2 rounded-lg px-2">
        <div className="flex items-center justify-between mb-2">
          <div>
            <h4 className="text-sm font-medium text-white">
              自动清除历史记录
            </h4>
            <p className="text-xs text-gray-400 mt-0.5">
              定期自动清除剪贴板历史记录
            </p>
          </div>
          <div className="w-36">
            <select className="w-full bg-gray-700 border border-gray-700 rounded-lg px-3 py-1.5 text-sm text-white focus:outline-none focus:ring-1 focus:ring-violet-400">
              <option>从不</option>
              <option>每天</option>
              <option>每周</option>
              <option>每月</option>
              <option>每次退出</option>
            </select>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SecuritySection;
