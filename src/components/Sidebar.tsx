import React from "react";
import { Link, useLocation } from "react-router-dom";

const Sidebar: React.FC = () => {
  const location = useLocation();
  const path = location.pathname;

  return (
    <div className="w-16 bg-gray-900 border-r border-gray-800/50">
      <div className="flex flex-col h-full">
        {/* 顶部导航图标 */}
        <div className="pt-4 px-2 space-y-4">
          {/* 仪表板 */}
          <Link
            to="/"
            className={`group flex items-center justify-center px-2 py-3 text-sm font-medium rounded-lg ${
              path === "/"
                ? "bg-gray-600 bg-opacity-80 text-white"
                : "text-gray-300 hover:bg-gray-600 hover:bg-opacity-60 hover:text-white transition duration-150"
            }`}
            title="仪表板"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className={`h-6 w-6 ${
                path === "/"
                  ? "text-violet-300"
                  : "text-gray-400 group-hover:text-violet-300"
              }`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"
              />
            </svg>
          </Link>

          {/* 历史记录 */}
          <Link
            to="/history"
            className={`group flex items-center justify-center px-2 py-3 text-sm font-medium rounded-lg ${
              path === "/history"
                ? "bg-gray-600 bg-opacity-80 text-white"
                : "text-gray-300 hover:bg-gray-600 hover:bg-opacity-60 hover:text-white transition duration-150"
            }`}
            title="历史记录"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className={`h-6 w-6 ${
                path === "/history"
                  ? "text-violet-300"
                  : "text-gray-400 group-hover:text-violet-300"
              }`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </Link>

          {/* 设备管理 */}
          <Link
            to="/devices"
            className={`group flex items-center justify-center px-2 py-3 text-sm font-medium rounded-lg ${
              path === "/devices"
                ? "bg-gray-600 bg-opacity-80 text-white"
                : "text-gray-300 hover:bg-gray-600 hover:bg-opacity-60 hover:text-white transition duration-150"
            }`}
            title="设备管理"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className={`h-6 w-6 ${
                path === "/devices"
                  ? "text-violet-300"
                  : "text-gray-400 group-hover:text-violet-300"
              }`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"
              />
            </svg>
          </Link>
        </div>

        {/* 底部导航 - 设置 */}
        <div className="mt-auto pb-4 px-2">
          <Link
            to="/settings"
            className={`group flex items-center justify-center px-2 py-3 text-sm font-medium rounded-lg ${
              path === "/settings"
                ? "bg-gray-600 bg-opacity-80 text-white"
                : "text-gray-300 hover:bg-gray-600 hover:bg-opacity-60 hover:text-white transition duration-150"
            }`}
            title="设置"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className={`h-6 w-6 ${
                path === "/settings"
                  ? "text-violet-300"
                  : "text-gray-400 group-hover:text-violet-300"
              }`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
              />
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
              />
            </svg>
          </Link>
        </div>
      </div>
    </div>
  );
};

export default Sidebar;
