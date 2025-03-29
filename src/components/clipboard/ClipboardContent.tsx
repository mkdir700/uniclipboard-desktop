import React, { useState, useEffect } from "react";
import ClipboardItem from "./ClipboardItem";
import {
  getDisplayType,
  isImageType,
  ClipboardItemResponse,
  Filter,
} from "@/api/clipboardItems";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import {
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

  // 本地状态用于转换后的显示项目
  const [clipboardItems, setClipboardItems] = useState<DisplayClipboardItem[]>(
    []
  );

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
