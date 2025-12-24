import React, { useState, useEffect } from "react";
import { Inbox } from "lucide-react";
import ClipboardItem from "./ClipboardItem";
import {
  getDisplayType,
  ClipboardItemResponse,
  Filter,
  ClipboardTextItem,
  ClipboardImageItem,
  ClipboardLinkItem,
  ClipboardCodeItem,
  ClipboardFileItem,
} from "@/api/clipboardItems";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import {
  removeClipboardItem,
  copyToClipboard,
  clearError as clearReduxError,
  toggleFavoriteItem,
} from "@/store/slices/clipboardSlice";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Skeleton } from "@/components/ui/skeleton";

interface DisplayClipboardItem {
  id: string;
  type: "text" | "image" | "link" | "code" | "file" | "unknown";
  time: string;
  isDownloaded?: boolean;
  isFavorited?: boolean;
  content:
    | ClipboardTextItem
    | ClipboardImageItem
    | ClipboardLinkItem
    | ClipboardCodeItem
    | ClipboardFileItem
    | null;
  device?: string;
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
    console.log("转换为显示项目:", item);
    // 获取适合UI显示的类型
    const type = getDisplayType(item.item);

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

    // 创建显示项目
    return {
      id: item.id,
      type,
      time: timeString,
      isDownloaded: item.is_downloaded,
      isFavorited: item.is_favorited,
      content:
        type === "text"
          ? item.item.text
          : type === "image"
          ? item.item.image
          : type === "link"
          ? item.item.link
          : type === "code"
          ? item.item.code
          : type === "file"
          ? item.item.file
          : null,
      device: item.device_id,
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

  // 骨架屏加载状态
  if (loading && clipboardItems.length === 0) {
    return (
      <div className="flex-1 overflow-y-auto scrollbar-thin px-8 pb-24">
        <div className="bg-muted/30 rounded-2xl p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {Array.from({ length: 6 }).map((_, i) => (
              <Skeleton key={i} className="h-56 w-full rounded-2xl" />
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto scrollbar-thin px-8 pb-24">
      {error && (
        <Alert variant="destructive" className="mx-8 mt-4">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      <div className="bg-muted/30 rounded-2xl p-6">
        {clipboardItems.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {clipboardItems.map((item) => (
              <ClipboardItem
                key={item.id}
                type={item.type}
                time={item.time}
                device={item.device}
                content={item.content}
                isDownloaded={item.isDownloaded}
                isFavorited={item.isFavorited}
                onDelete={() => handleDeleteItem(item.id)}
                onCopy={() => handleCopyItem(item.id)}
                toggleFavorite={(isFavorited) =>
                  handleToggleFavorite(item.id, isFavorited)
                }
              />
            ))}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
            <Inbox className="h-16 w-16 mb-4 opacity-50" />
            <p className="text-lg font-medium">没有剪贴板项</p>
            <p className="text-sm mt-1">复制内容后将显示在这里</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default ClipboardContent;
