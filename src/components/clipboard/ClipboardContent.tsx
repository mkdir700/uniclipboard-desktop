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

  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  const handleToggleSelect = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  // 骨架屏加载状态
  if (loading && clipboardItems.length === 0) {
    return (
      <div className="h-full overflow-y-auto scrollbar-thin px-4 pb-32 pt-2">
        <div className="flex flex-col gap-1">
          {Array.from({ length: 12 }).map((_, i) => (
            <Skeleton key={i} className="h-12 w-full rounded-lg" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto scrollbar-thin px-4 pb-32 pt-2">
      {error && (
        <Alert variant="destructive" className="mb-4 mx-1">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {clipboardItems.length > 0 ? (
        <div className="flex flex-col">
          {clipboardItems.map((item, index) => (
            <ClipboardItem
              key={item.id}
              index={index + 1}
              type={item.type}
              time={item.time}
              device={item.device}
              content={item.content}
              isDownloaded={item.isDownloaded}
              isFavorited={item.isFavorited}
              isSelected={selectedIds.has(item.id)}
              onSelect={() => handleToggleSelect(item.id)}
              onDelete={() => handleDeleteItem(item.id)}
              onCopy={() => handleCopyItem(item.id)}
              toggleFavorite={(isFavorited) =>
                handleToggleFavorite(item.id, isFavorited)
              }
            />
          ))}
        </div>
      ) : (
        <div className="h-full flex flex-col items-center justify-center -mt-20">
          <div className="relative mb-6">
            <div className="absolute inset-0 bg-primary/20 blur-2xl rounded-full" />
            <div className="relative bg-card p-6 rounded-3xl border border-border/50 shadow-xl">
               <Inbox className="h-12 w-12 text-primary/80" />
            </div>
          </div>
          <h3 className="text-xl font-bold text-foreground mb-2">没有剪贴板项</h3>
          <p className="text-muted-foreground text-center max-w-xs">
            当您复制内容时，它们会显示在这里。
            <br />
            试着复制一些文本或图片吧！
          </p>
        </div>
      )}
    </div>
  );
};

export default ClipboardContent;
