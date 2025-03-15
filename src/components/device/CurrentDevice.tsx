import React from "react";

const CurrentDevice: React.FC = () => {
  return (
    <>
      <div className="mb-6">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-sm font-medium text-gray-400">当前设备</h3>
          <div className="flex-grow ml-3 border-t border-gray-800/50"></div>
        </div>

        <div className="bg-gray-700/80 rounded-lg border border-violet-500/30 p-4 flex items-center justify-between shadow-md">
          <div className="flex items-center">
            <div className="flex-shrink-0 h-12 w-12 bg-violet-500/20 rounded-lg flex items-center justify-center">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-6 w-6 text-violet-300"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
                />
              </svg>
            </div>
            <div className="ml-4">
              <div className="flex items-center">
                <h4 className="font-medium text-white">MacBook Pro</h4>
                <span className="ml-2 text-xs px-2 py-0.5 bg-violet-500/30 rounded-full text-violet-300 font-medium">
                  当前设备
                </span>
              </div>
              <p className="text-xs text-gray-400 mt-1">最后活动时间: 现在</p>
            </div>
          </div>
          <div className="flex items-center space-x-3">
            <span className="text-xs px-2 py-1 bg-green-500/20 rounded-full text-green-400 flex items-center">
              <div className="h-1.5 w-1.5 rounded-full bg-green-500 mr-1 animate-pulse"></div>
              在线
            </span>
            <button className="text-gray-400 hover:text-white">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-5 w-5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                />
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </>
  );
};

export default CurrentDevice;
