import React, { useState } from "react";

const Header: React.FC = () => {
  // 定义筛选类型
  const filterTypes = [
    { id: "all", label: "全部", icon: "📋" },
    { id: "favorite", label: "收藏", icon: "⭐" },
    { id: "text", label: "文本", icon: "📝" },
    { id: "image", label: "图片", icon: "🖼️" },
    { id: "link", label: "链接", icon: "🔗" },
    { id: "file", label: "文件", icon: "📁" },
    { id: "code", label: "代码", icon: "💻" },
  ];

  // 当前选中的筛选类型
  const [activeFilter, setActiveFilter] = useState("all");

  // 处理筛选器点击
  const handleFilterClick = (filterId: string) => {
    setActiveFilter(filterId);
    // 这里可以添加筛选逻辑，例如触发父组件的回调函数
    // onFilterChange(filterId);
  };

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
        </div>
      </div>

      {/* 内容类型筛选器 */}
      <div className="px-4 pb-3 overflow-x-auto">
        <div className="flex space-x-2 text-sm">
          {filterTypes.map((filter) => (
            <button
              key={filter.id}
              className={`px-3 py-1.5 rounded-md transition-all duration-200 flex items-center ${
                activeFilter === filter.id
                  ? filter.id === "favorite"
                    ? "bg-yellow-500 text-white shadow-lg shadow-yellow-500/20 transform scale-105"
                    : "bg-violet-500 text-white shadow-lg shadow-violet-500/20 transform scale-105"
                  : "bg-gray-800 hover:bg-gray-700 text-gray-300 hover:text-white"
              }`}
              onClick={() => handleFilterClick(filter.id)}
            >
              <span className="mr-1.5">{filter.icon}</span>
              <span>{filter.label}</span>
            </button>
          ))}
        </div>
      </div>
    </header>
  );
};

export default Header;
