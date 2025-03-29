import React, { useState, useEffect, useRef, useCallback } from "react";
import ClipboardItem from "./ClipboardItem";
import {
  getDisplayType,
  isImageType,
  ClipboardItemResponse,
  OrderBy,
  Filter,
} from "@/api/clipboardItems";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import {
  fetchClipboardItems,
  removeClipboardItem,
  copyToClipboard,
  clearError as clearReduxError,
  toggleFavoriteItem,
} from "@/store/slices/clipboardSlice";

interface DisplayClipboardItem {
  id: string;
  type: "text" | "image" | "link" | "code" | "file";
  content: string;
  contentSize: number;
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
  lastEventTimestamp?: number;
  eventHandler?: (specificFilter: Filter) => void;
}

const globalListenerState: ListenerState = {
  isActive: false,
};

// 防抖延迟时间（毫秒）
const DEBOUNCE_DELAY = 500;

interface ClipboardContentProps {
  filter: Filter;
}

const ClipboardContent: React.FC<ClipboardContentProps> = ({ filter }) => {
  // 使用 Redux 状态和 dispatch
  const dispatch = useAppDispatch();
  const {
    items: reduxItems,
    loading,
    error,
  } = useAppSelector((state) => state.clipboard);

  // 使用ref保存最新的filter值，明确类型
  const currentFilterRef = useRef<Filter>(filter);

  // 使用useCallback包装loadClipboardRecords，增加防抖处理
  const debouncedLoadRef = useRef<number | null>(null);

  const loadClipboardRecords = useCallback(
    async (specificFilter?: Filter) => {
      const filterToUse = specificFilter || currentFilterRef.current;
      console.log("开始加载剪贴板记录...", filterToUse);
      dispatch(
        fetchClipboardItems({
          orderBy: OrderBy.ActiveTimeDesc,
          filter: filterToUse,
        })
      );
    },
    [dispatch]
  );

  const debouncedLoadClipboardRecords = useCallback(
    (specificFilter?: Filter) => {
      // 清除之前的定时器
      if (debouncedLoadRef.current) {
        clearTimeout(debouncedLoadRef.current);
      }

      // 设置新的定时器
      debouncedLoadRef.current = setTimeout(() => {
        loadClipboardRecords(specificFilter);
        debouncedLoadRef.current = null;
      }, DEBOUNCE_DELAY);
    },
    [loadClipboardRecords]
  );

  // 更新ref以跟踪最新的filter
  useEffect(() => {
    console.log("filter变化，更新ref值:", filter);
    currentFilterRef.current = filter;
    loadClipboardRecords(filter); // 直接加载，不用防抖
    console.log("filter变化，更新ref值:", currentFilterRef.current);
  }, [filter, loadClipboardRecords]);

  // 本地状态用于转换后的显示项目
  const [clipboardItems, setClipboardItems] = useState<DisplayClipboardItem[]>(
    []
  );

  // 加载剪贴板记录
  useEffect(() => {
    // 更新全局事件处理函数，使其总是使用最新的过滤器和防抖函数
    globalListenerState.eventHandler = (specificFilter: Filter) => {
      debouncedLoadClipboardRecords(specificFilter);
    };

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
          // 清除之前可能存在的监听器
          if (globalListenerState.unlisten) {
            console.log("清除之前的监听器");
            globalListenerState.unlisten();
            globalListenerState.unlisten = undefined;
          }

          // 使用listen函数监听全局事件
          const unlisten = await listen<{
            record_id: string;
            timestamp: number;
          }>("clipboard-new-content", (event) => {
            console.log("收到新剪贴板内容事件:", event);

            // 检查事件时间戳，避免短时间内重复处理同一事件
            const currentTime = Date.now();
            if (
              globalListenerState.lastEventTimestamp &&
              currentTime - globalListenerState.lastEventTimestamp <
                DEBOUNCE_DELAY
            ) {
              console.log("忽略短时间内的重复事件");
              return;
            }

            // 更新最后事件时间戳
            globalListenerState.lastEventTimestamp = currentTime;

            // 使用最新的事件处理函数
            if (globalListenerState.eventHandler) {
              globalListenerState.eventHandler(currentFilterRef.current);
            }
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
      // 清除防抖定时器
      if (debouncedLoadRef.current) {
        clearTimeout(debouncedLoadRef.current);
      }
      // 不清理全局监听器，让它持续存在
      console.log("组件卸载，但保持全局监听器活跃");
    };
  }, [debouncedLoadClipboardRecords]);

  // 监听 Redux 中的 items 变化，转换为显示项目
  useEffect(() => {
    console.log("筛选条件:", filter);
    console.log("查询结果:", reduxItems);

    if (reduxItems && reduxItems.length > 0) {
      const items: DisplayClipboardItem[] =
        reduxItems.map(convertToDisplayItem);
      setClipboardItems(items);
      console.log("转换后的显示项目:", items);
    } else {
      setClipboardItems([]);
      console.log("没有查询到任何项目");
    }
  }, [reduxItems, filter]);

  // 将剪贴板项目转换为显示项目
  const convertToDisplayItem = (
    item: ClipboardItemResponse
  ): DisplayClipboardItem => {
    // 获取适合UI显示的类型
    const type = getDisplayType(item.content_type);

    // 格式化时间
    const activeTime = new Date(item.active_time * 1000); // 转换为毫秒
    const now = new Date();
    const diffMs = now.getTime() - activeTime.getTime();
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
      contentSize: item.content_size,
      time: timeString,
      device: item.device_id,
      imageUrl,
      isDownloaded: item.is_downloaded,
      isFavorited: item.is_favorited,
    };
  };

  // 处理删除剪贴板项
  const handleDeleteItem = async (id: string) => {
    dispatch(removeClipboardItem(id));
  };

  // 处理复制到剪贴板
  const handleCopyItem = async (itemId: string) => {
    try {
      console.log(`复制项目 ID: ${itemId}`);
      const result = await dispatch(copyToClipboard(itemId)).unwrap();

      // 注意：不需要手动调用loadClipboardRecords，
      // 因为后端会发送clipboard-new-content事件，
      // 事件监听器会自动处理加载

      return result.success;
    } catch (err) {
      console.error("复制到剪贴板失败", err);
      return false;
    }
  };

  const handleToggleFavorite = async (itemId: string, isFavorited: boolean) => {
    dispatch(toggleFavoriteItem({ id: itemId, isFavorited }));
  };

  // 清除错误信息
  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => {
        dispatch(clearReduxError());
      }, 3000);
      return () => clearTimeout(timer);
    }
  }, [error, dispatch]);

  if (loading && clipboardItems.length === 0) {
    return (
      <div className="flex-1 overflow-hidden flex items-center justify-center">
        <div className="text-gray-500">加载中...</div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-hidden">
      {error && (
        <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-2 rounded mx-4 mt-2">
          {error}
        </div>
      )}
      <div className="h-full overflow-y-auto hide-scrollbar px-4 py-4">
        <div className="space-y-4">
          <div className="space-y-3">
            {clipboardItems.length > 0 ? (
              clipboardItems.map((item) => (
                <ClipboardItem
                  key={item.id}
                  type={item.type}
                  content={item.content}
                  fileSize={item.contentSize}
                  time={item.time}
                  device={item.device}
                  imageUrl={item.imageUrl}
                  isDownloaded={item.isDownloaded}
                  isFavorited={item.isFavorited}
                  onDelete={() => handleDeleteItem(item.id)}
                  onCopy={() => handleCopyItem(item.id)}
                  toggleFavorite={(isFavorited) =>
                    handleToggleFavorite(item.id, isFavorited)
                  }
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
