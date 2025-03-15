import React from "react";

const HistoryHeader: React.FC = () => {
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
              placeholder="搜索历史记录..."
            />
          </div>
        </div>

        <div className="ml-4 flex items-center space-x-3">
          {/* 时间范围选择器 */}
          <div className="relative">
            <select className="appearance-none bg-gray-800 text-sm border border-gray-700/40 rounded-lg px-3 py-2 text-gray-300 focus:outline-none focus:ring-2 focus:ring-violet-300/40 focus:border-transparent">
              <option>全部时间</option>
              <option>最近7天</option>
              <option>最近30天</option>
              <option>最近3个月</option>
              <option>自定义范围</option>
            </select>
            <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-400">
              <svg
                className="h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M19 9l-7 7-7-7"
                />
              </svg>
            </div>
          </div>

          {/* 导出按钮 */}
          <button className="bg-gray-800 text-gray-300 hover:text-white px-3 py-2 rounded-lg text-sm border border-gray-700/40 flex items-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-4 w-4 mr-1"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0l-4 4m4-4v12"
              />
            </svg>
            <span>导出</span>
          </button>
        </div>
      </div>
    </header>
  );
};

export default HistoryHeader;
