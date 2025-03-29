import React from "react";
import { ClipboardStats } from "@/api/clipboardItems";
import { formatFileSize } from "@/utils";
import { useAppDispatch } from "@/store/hooks";
import { clearAllItems } from "@/store/slices/clipboardSlice";
import { syncClipboardItems } from "@/api/clipboardItems";

interface ActionBarProps {
  stats: ClipboardStats;
  onSync?: () => void;
}

const ActionBar: React.FC<ActionBarProps> = ({ stats, onSync }) => {
  const dispatch = useAppDispatch();

  // 处理清理所有剪贴板项
  const handleClearAll = async () => {
    if (window.confirm("确定要清理所有剪贴板项吗？")) {
      try {
        await dispatch(clearAllItems()).unwrap();
      } catch (err) {
        console.error("清理剪贴板项失败:", err);
      }
    }
  };

  // 处理立即同步
  const handleSync = async () => {
    try {
      console.log("开始同步剪贴板项...");
      await syncClipboardItems();
      console.log("剪贴板项同步完成");

      // 调用父组件传递的同步成功回调
      if (onSync) {
        onSync();
      }
    } catch (err) {
      console.error("同步剪贴板项失败:", err);
      alert("同步失败，请稍后重试。");
    }
  };

  return (
    <footer className="bg-gray-900 border-t border-gray-800/50 px-4 py-3">
      <div className="flex items-center justify-between">
        <div className="flex space-x-2">
          <button className="flex items-center justify-center h-9 w-9 rounded-full bg-gray-800 hover:bg-gray-700 text-gray-400 hover:text-white transition-colors">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-5 w-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M12 6v6m0 0v6m0-6h6m-6 0H6"
              />
            </svg>
          </button>
          <button className="flex items-center justify-center h-9 w-9 rounded-full bg-gray-800 hover:bg-gray-700 text-gray-400 hover:text-white transition-colors">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-5 w-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
              />
            </svg>
          </button>
          <button className="flex items-center justify-center h-9 w-9 rounded-full bg-gray-800 hover:bg-gray-700 text-gray-400 hover:text-white transition-colors">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-5 w-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
              />
            </svg>
          </button>
        </div>

        <div className="text-xs text-gray-400">
          <span>
            共 {stats.total_items} 项 · 已使用{" "}
            {formatFileSize(stats.total_size)}
          </span>
        </div>

        <div className="flex space-x-2">
          <button
            className="px-3 py-2 bg-gray-800 rounded text-sm text-gray-300 hover:bg-gray-700 transition-colors"
            onClick={handleClearAll}
          >
            清理所有
          </button>
          <button
            className="px-3 py-2 bg-violet-500 rounded text-sm text-white hover:bg-violet-400 transition-colors"
            onClick={handleSync}
          >
            立即同步
          </button>
        </div>
      </div>
    </footer>
  );
};

export default ActionBar;
