import React, { useMemo, useState, useEffect } from "react";
import { Inbox } from "lucide-react";
import ClipboardItem from "./ClipboardItem";
import ClipboardSelectionActionBar from "./ClipboardSelectionActionBar";
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

  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [lastSelectedIndex, setLastSelectedIndex] = useState<number | null>(null);

  // 监听 Redux 中的 items 变化，转换为显示项目
  useEffect(() => {
    console.log("筛选条件:", filter);
    console.log("查询结果:", reduxItems);

    if (reduxItems && reduxItems.length > 0) {
      let items: DisplayClipboardItem[] = reduxItems.map(convertToDisplayItem);
      if (filter === Filter.Favorited) {
        items = items.filter((it) => it.isFavorited);
      }
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

  // 清除错误信息
  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => {
        dispatch(clearReduxError());
      }, 3000);
      return () => clearTimeout(timer);
    }
  }, [error, dispatch]);

  // 当列表变化时，清理已经不存在的选择
  useEffect(() => {
    setSelectedIds((prev) => {
      if (prev.size === 0) return prev;
      const valid = new Set(clipboardItems.map((i) => i.id));
      const next = new Set<string>();
      for (const id of prev) {
        if (valid.has(id)) next.add(id);
      }
      return next;
    });
    if (lastSelectedIndex !== null && lastSelectedIndex >= clipboardItems.length) {
      setLastSelectedIndex(null);
    }
  }, [clipboardItems, lastSelectedIndex]);

  const handleSelect = (
    id: string,
    index: number,
    event: React.MouseEvent<HTMLDivElement>
  ) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);

      const isMultiToggle = event.metaKey || event.ctrlKey;
      const isRange = event.shiftKey;

      if (isRange && lastSelectedIndex !== null) {
        const start = Math.min(lastSelectedIndex, index);
        const end = Math.max(lastSelectedIndex, index);
        const rangeIds = clipboardItems.slice(start, end + 1).map((i) => i.id);

        if (!isMultiToggle) next.clear();
        for (const rid of rangeIds) next.add(rid);
        return next;
      }

      if (isMultiToggle) {
        if (next.has(id)) next.delete(id);
        else next.add(id);
        return next;
      }

      // 默认：单选
      if (next.size === 1 && next.has(id)) return next;
      next.clear();
      next.add(id);
      return next;
    });

    setLastSelectedIndex(index);
  };

  const selectedItems = useMemo(() => {
    if (selectedIds.size === 0) return [];
    return clipboardItems.filter((it) => selectedIds.has(it.id));
  }, [clipboardItems, selectedIds]);

  const favoriteIntent: "favorite" | "unfavorite" = useMemo(() => {
    if (selectedItems.length === 0) return "favorite";
    const allFavorited = selectedItems.every((it) => Boolean(it.isFavorited));
    return allFavorited ? "unfavorite" : "favorite";
  }, [selectedItems]);

  const handleBatchCopy = async (): Promise<boolean> => {
    if (selectedItems.length === 0) return false;
    if (selectedItems.length === 1) {
      return handleCopyItem(selectedItems[0].id);
    }

    const toText = (it: DisplayClipboardItem): string => {
      switch (it.type) {
        case "text":
          return (it.content as ClipboardTextItem)?.display_text ?? "";
        case "code":
          return (it.content as ClipboardCodeItem)?.code ?? "";
        case "link":
          return (it.content as ClipboardLinkItem)?.url ?? "";
        case "file": {
          const names = (it.content as ClipboardFileItem)?.file_names ?? [];
          return names.length > 0 ? names.join("\n") : "[文件]";
        }
        case "image":
          return "[图片]";
        default:
          return "";
      }
    };

    const text = selectedItems.map(toText).filter(Boolean).join("\n\n");
    if (!text) return false;
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch (e) {
      console.error("批量复制失败:", e);
      return false;
    }
  };

  const handleBatchToggleFavorite = async () => {
    if (selectedItems.length === 0) return;
    const targetFavorited = favoriteIntent === "favorite";
    await Promise.all(
      selectedItems.map((it) =>
        dispatch(toggleFavoriteItem({ id: it.id, isFavorited: targetFavorited }))
          .unwrap()
          .catch((e) => console.error("设置收藏状态失败:", e))
      )
    );
  };

  const handleBatchDelete = async () => {
    if (selectedItems.length === 0) return;
    const count = selectedItems.length;
    const ok = window.confirm(`确定要删除选中的 ${count} 项吗？`);
    if (!ok) return;

    for (const it of selectedItems) {
      try {
        await dispatch(removeClipboardItem(it.id)).unwrap();
      } catch (e) {
        console.error("删除失败:", e);
      }
    }
    setSelectedIds(new Set());
    setLastSelectedIndex(null);
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
    <div className="h-full relative">
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
              isSelected={selectedIds.has(item.id)}
              onSelect={(e) => handleSelect(item.id, index, e)}
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

      <ClipboardSelectionActionBar
        selectedCount={selectedIds.size}
        favoriteIntent={favoriteIntent}
        onCopy={handleBatchCopy}
        onToggleFavorite={handleBatchToggleFavorite}
        onDelete={handleBatchDelete}
        onClearSelection={() => {
          setSelectedIds(new Set());
          setLastSelectedIndex(null);
        }}
      />
    </div>
  );
};

export default ClipboardContent;
