import React from 'react'

const SettingFooter: React.FC = () => {
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
                />
              </svg>
              <span>重置所有设置</span>
            </button>
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
                  d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              <span>帮助</span>
            </button>
          </div>

          <div className="flex space-x-2">
            <button className="px-3 py-2 bg-gray-800 rounded text-sm text-gray-300 hover:bg-gray-700 transition-colors">
              取消
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
                  d="M5 13l4 4L19 7"
                />
              </svg>
              <span>应用设置</span>
            </button>
          </div>
        </div>
      </footer>
    </>
  )
}

export default SettingFooter
