import React from "react";

const Header: React.FC = () => {
  return (
    <header className="bg-gray-900 border-b border-gray-800/50">
      <div className="px-4 py-3 flex items-center justify-between">
        <div className="flex-1 flex">
          <div className="w-full max-w-lg relative">
            <span className="absolute inset-y-0 left-0 pl-3 flex items-center">
              <svg
                className="h-5 w-5 text-gray-400"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
            </span>
            <input
              type="text"
              className="block w-full bg-gray-800 text-sm border border-gray-700/40 rounded-lg pl-10 pr-4 py-2 text-gray-300 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-300/40 focus:border-transparent"
              placeholder="搜索剪切板内容..."
            />
          </div>
        </div>

        <div className="ml-4 flex items-center space-x-4">
          {/* 同步状态指示器 */}
          <div className="bg-green-500/20 px-3 py-1 rounded-full flex items-center">
            <div className="h-2 w-2 rounded-full bg-green-500 mr-2 animate-pulse"></div>
            <span className="text-xs text-green-300">已同步</span>
          </div>

          {/* 通知图标 */}
          <button className="text-gray-400 hover:text-white focus:outline-none">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-6 w-6"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
              />
            </svg>
          </button>
        </div>
      </div>

      {/* 内容类型筛选器 */}
      <div className="px-4 pb-3 flex space-x-1 text-sm">
        <button className="px-3 py-1 bg-violet-500/90 rounded-md text-white">
          全部
        </button>
        <button className="px-3 py-1 bg-gray-700/70 hover:bg-gray-700/90 rounded-md text-gray-300">
          文本
        </button>
        <button className="px-3 py-1 bg-gray-700/70 hover:bg-gray-700/90 rounded-md text-gray-300">
          图片
        </button>
        <button className="px-3 py-1 bg-gray-700/70 hover:bg-gray-700/90 rounded-md text-gray-300">
          链接
        </button>
        <button className="px-3 py-1 bg-gray-700/70 hover:bg-gray-700/90 rounded-md text-gray-300">
          文件
        </button>
        <button className="px-3 py-1 bg-gray-700/70 hover:bg-gray-700/90 rounded-md text-gray-300">
          代码
        </button>
      </div>
    </header>
  );
};

export default Header;
