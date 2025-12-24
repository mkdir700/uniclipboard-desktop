import React from "react";
import { RefreshCw } from "lucide-react";
import { ClipboardStats } from "@/api/clipboardItems";
import { formatFileSize } from "@/utils";
import { useAppDispatch } from "@/store/hooks";
import { clearAllItems } from "@/store/slices/clipboardSlice";
import { syncClipboardItems } from "@/api/clipboardItems";
import { Button } from "@/components/ui/button";

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
    <footer className="absolute bottom-0 w-full glass-strong border-t border-border px-8 py-4 flex items-center justify-between z-10">
      <div className="text-sm text-muted-foreground flex items-center gap-2">
        <span className="font-medium text-foreground">已同步 {stats.total_items} 项</span>
        <span>·</span>
        <span>已使用 {formatFileSize(stats.total_size)}</span>
      </div>

      <div className="flex items-center gap-3">
        <Button
          variant="outline"
          size="sm"
          onClick={handleClearAll}
          className="rounded-lg"
        >
          清理所有
        </Button>
        <Button
          size="sm"
          onClick={handleSync}
          className="rounded-lg bg-primary hover:bg-primary/90 shadow-lg shadow-primary/30"
        >
          <RefreshCw className="h-4 w-4 mr-2" />
          立即同步
        </Button>
      </div>
    </footer>
  );
};

export default ActionBar;
