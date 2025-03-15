import React from "react";

interface HeaderProps {
  addDevice: () => void;
}

const Header: React.FC<HeaderProps> = ({ addDevice }) => {
  return (
    <>
      {" "}
      <header className="bg-gray-900 border-b border-gray-800/50">
        <div className="px-4 py-4 flex items-center justify-between">
          <h1 className="text-xl font-semibold text-white">设备管理</h1>

          <div className="flex items-center space-x-3">
            {/* 添加设备按钮 */}
            <button
              onClick={addDevice}
              className="bg-violet-500 hover:bg-violet-400 text-white px-4 py-2 rounded-lg text-sm font-medium transition duration-150 flex items-center"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-5 w-5 mr-1.5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 6v6m0 0v6m0-6h6m-6 0H6"
                />
              </svg>
              添加新设备
            </button>
          </div>
        </div>

        {/* 子导航 */}
        <div className="px-4 pb-2 flex space-x-4 text-sm">
          <button className="text-white border-b-2 border-violet-400 pb-2 font-medium">
            已连接设备
          </button>
          <button className="text-gray-400 hover:text-white pb-2 border-b-2 border-transparent">
            配对请求
          </button>
          <button className="text-gray-400 hover:text-white pb-2 border-b-2 border-transparent">
            全局规则
          </button>
        </div>
      </header>
    </>
  );
};

export default Header;
