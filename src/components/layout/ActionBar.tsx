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
        <div className="flex space-x-2 text-xs text-gray-400">
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
