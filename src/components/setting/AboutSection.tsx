import React from "react";

const AboutSection: React.FC = () => {
  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center">
          <div className="h-10 w-10 rounded-lg bg-gradient-to-br from-violet-400 to-violet-300 flex items-center justify-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-6 w-6 text-white"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
              />
            </svg>
          </div>
          <div className="ml-3">
            <h4 className="text-white font-medium">ClipSync</h4>
            <p className="text-xs text-gray-400">版本 2.4.1</p>
          </div>
        </div>
        <button className="px-3 py-1.5 bg-gray-700 hover:bg-gray-600 text-sm text-gray-300 rounded-lg transition duration-150">
          检查更新
        </button>
      </div>

      <div className="space-y-2 text-sm text-gray-400">
        <p> 2023 ClipSync Team. </p>
        <div className="flex space-x-4">
          <a href="#" className="text-violet-400 hover:text-violet-300">
            隐私政策
          </a>
          <a href="#" className="text-violet-400 hover:text-violet-300">
            使用条款
          </a>
          <a href="#" className="text-violet-400 hover:text-violet-300">
            帮助中心
          </a>
        </div>
      </div>
    </div>
  );
};

export default AboutSection;
