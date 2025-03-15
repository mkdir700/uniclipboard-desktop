import React from "react";

const Footer: React.FC = () => {
  return (
    <>
      <footer className="bg-gray-900 border-t border-gray-800/50 px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <button className="px-3 py-2 bg-gray-800 rounded text-sm text-gray-300 hover:bg-gray-700 transition-colors flex items-center">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-4 w-4 mr-1"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                ></path>
              </svg>
              <span>全部重新连接</span>
            </button>
          </div>

          <div className="text-xs text-gray-400">
            <span>设备总数: 4 | 在线设备: 2</span>
          </div>

          <div className="flex space-x-2">
            <button className="px-3 py-2 bg-gray-800 rounded text-sm text-gray-300 hover:bg-gray-700 transition-colors">
              设备状态诊断
            </button>
            <button className="px-3 py-2 bg-violet-500 rounded text-sm text-white hover:bg-violet-400 transition-colors flex items-center">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-4 w-4 mr-1"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
                ></path>
              </svg>
              <span>应用同步规则</span>
            </button>
          </div>
        </div>
      </footer>
    </>
  );
};

export default Footer;
