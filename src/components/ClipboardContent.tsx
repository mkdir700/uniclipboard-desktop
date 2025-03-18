import React, { useState, useEffect } from "react";
import ClipboardItem from "./ClipboardItem";
import {
  getClipboardItems,
  deleteClipboardItem,
  getDisplayType,
  isImageType,
  ClipboardItemResponse,
} from "../api/clipboardItems";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface DisplayClipboardItem {
  id: string;
  type: "text" | "image" | "link" | "code" | "file";
  content: string;
  time: string;
  device?: string;
  imageUrl?: string;
  isDownloaded?: boolean;
  isFavorited?: boolean;
}

// 全局监听器状态管理
interface ListenerState {
  isActive: boolean;
  unlisten?: () => void;
  cleanupPromise?: Promise<() => void>;
}

const globalListenerState: ListenerState = {
  isActive: false
};

const ClipboardContent: React.FC = () => {
  // 剪贴板项目状态
  const [clipboardItems, setClipboardItems] = useState<DisplayClipboardItem[]>(
    []
  );
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 加载剪贴板记录
  useEffect(() => {
    // 先立即加载一次数据
    loadClipboardRecords();

    // 设置监听器的函数
    const setupListener = async () => {
      // 只有在还没有活跃的监听器时才设置
      if (!globalListenerState.isActive) {
        console.log("设置全局监听器...");
        globalListenerState.isActive = true;
        
        try {
          console.log("启动后端剪贴板新内容监听...");
          await invoke("listen_clipboard_new_content");
          console.log("后端剪贴板新内容监听已启动");
          
          console.log("开始监听剪贴板新内容事件...");
          // 使用listen函数监听全局事件
          const unlisten = await listen<{
            record_id: string;
            timestamp: number;
          }>("clipboard-new-content", (event) => {
            console.log("收到新剪贴板内容事件:", event);
            // 重新加载剪贴板记录
            loadClipboardRecords();
          });
          
          // 保存解除监听的函数到全局状态
          globalListenerState.unlisten = unlisten;
          
        } catch (err) {
          console.error("设置监听器失败:", err);
          globalListenerState.isActive = false;
        }
      } else {
        console.log("监听器已经处于活跃状态，跳过设置");
      }
    };
    
    // 如果还没有设置监听器，则设置
    if (!globalListenerState.isActive) {
      setupListener();
    } else {
      console.log("全局监听器已存在，无需再次设置");
    }
    
    // 组件卸载时的清理函数
    return () => {
      // 不在这里清理全局监听器，让它持续存在
      console.log("组件卸载，但保持全局监听器活跃");
    };
  }, []);

  // 从后端加载剪贴板记录
  const loadClipboardRecords = async () => {
    console.log("开始加载剪贴板记录...");
    setLoading(true);
    try {
      // 使用 clipboardItems.ts 中的 API 获取剪贴板记录
      const records = await getClipboardItems(20, 0);

      // 转换记录为显示项目
      const items: DisplayClipboardItem[] = records.map(convertToDisplayItem);

      setClipboardItems(items);
    } catch (err) {
      console.error("加载剪贴板记录失败", err);
      setError("加载剪贴板记录失败");
    } finally {
      setLoading(false);
    }
  };

  // 将剪贴板项目转换为显示项目
  const convertToDisplayItem = (
    item: ClipboardItemResponse
  ): DisplayClipboardItem => {
    // 获取适合UI显示的类型
    const type = getDisplayType(item.content_type);

    // 格式化时间
    const createdAt = new Date(item.created_at * 1000); // 转换为毫秒
    const now = new Date();
    const diffMs = now.getTime() - createdAt.getTime();
    const diffMins = Math.round(diffMs / 60000);

    let timeString: string;
    if (diffMins < 1) {
      timeString = "刚刚";
    } else if (diffMins < 60) {
      timeString = `${diffMins}分钟前`;
    } else if (diffMins < 1440) {
      timeString = `${Math.floor(diffMins / 60)}小时前`;
    } else {
      timeString = `${Math.floor(diffMins / 1440)}天前`;
    }

    // 处理图片URL
    let imageUrl = undefined;
    if (isImageType(item.content_type)) {
      imageUrl = item.display_content.startsWith("data:")
        ? item.display_content
        : `data:image/png;base64,${item.display_content}`;
    }

    // 创建显示项目
    return {
      id: item.id,
      type,
      content: item.display_content,
      time: timeString,
      device: item.device_id,
      imageUrl,
      isDownloaded: item.is_downloaded,
      isFavorited: item.is_favorited,
    };
  };

  // 处理删除剪贴板项
  const handleDeleteItem = async (id: string) => {
    try {
      // 使用 clipboardItems.ts 中的 API 删除记录
      const success = await deleteClipboardItem(id);

      if (success) {
        // 更新状态
        setClipboardItems((prevItems) =>
          prevItems.filter((item) => item.id !== id)
        );
      } else {
        setError("删除剪贴板项目失败");
      }
    } catch (err) {
      console.error("删除剪贴板项目失败", err);
      setError("删除剪贴板项目失败");
    }
  };

  // 处理复制到剪贴板（这个功能暂未在API中实现，可以后续添加）
  const handleCopyItem = async (itemId: string) => {
    try {
      // 这里可以实现调用复制到剪贴板的API
      // 可以使用 itemId 查找项目并复制
      console.log(`复制项目 ID: ${itemId}`);
      // 暂时返回成功
      return true;
    } catch (err) {
      console.error("复制到剪贴板失败", err);
      setError("复制到剪贴板失败");
      return false;
    }
  };

  if (loading) {
    return (
      <div className="flex-1 overflow-hidden flex items-center justify-center">
        <div className="text-gray-500">加载中...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex-1 overflow-hidden flex items-center justify-center">
        <div className="text-red-500">{error}</div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-hidden">
      <div className="h-full overflow-y-auto hide-scrollbar px-4 py-4">
        <div className="space-y-4">
          <div className="space-y-3">
            {clipboardItems.length > 0 ? (
              clipboardItems.map((item) => (
                <ClipboardItem
                  key={item.id}
                  type={item.type}
                  content={item.content}
                  time={item.time}
                  device={item.device}
                  imageUrl={item.imageUrl}
                  isDownloaded={item.isDownloaded}
                  isFavorited={item.isFavorited}
                  onDelete={() => handleDeleteItem(item.id)}
                  onCopy={() => handleCopyItem(item.id)}
                />
              ))
            ) : (
              <div className="text-gray-500 text-center py-4">没有剪贴板项</div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default ClipboardContent;
