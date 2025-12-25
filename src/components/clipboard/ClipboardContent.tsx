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
import { useTranslation } from "react-i18next";

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
  const { t } = useTranslation();

  // Use Redux state and dispatch
  const dispatch = useAppDispatch();
  const {
    items: reduxItems,
    loading,
    error,
  } = useAppSelector((state) => state.clipboard);

  // Local state for converted display items
  const [clipboardItems, setClipboardItems] = useState<DisplayClipboardItem[]>(
    []
  );

  // Listen for changes in Redux items, convert to display items
  useEffect(() => {
    console.log(t("clipboard.content.filterCondition"), filter);
    console.log(t("clipboard.content.queryResults"), reduxItems);

    if (reduxItems && reduxItems.length > 0) {
      const items: DisplayClipboardItem[] =
        reduxItems.map(convertToDisplayItem);
      setClipboardItems(items);
      console.log(t("clipboard.content.convertedItems"), items);
    } else {
      setClipboardItems([]);
      console.log(t("clipboard.content.noItemsFound"));
    }
  }, [reduxItems, filter, t]);

  // Convert clipboard item to display item
  const convertToDisplayItem = (
    item: ClipboardItemResponse
  ): DisplayClipboardItem => {
    console.log(t("clipboard.content.logs.convertingItem"), item);
    // Get type suitable for UI display
    const type = getDisplayType(item.item);

    // Format time
    const activeTime = new Date(item.active_time * 1000); // Convert to milliseconds
    const now = new Date();
    const diffMs = now.getTime() - activeTime.getTime();
    const diffMins = Math.round(diffMs / 60000);

    let timeString: string;
    if (diffMins < 1) {
      timeString = t("clipboard.time.justNow");
    } else if (diffMins < 60) {
      timeString = t("clipboard.time.minutesAgo", { minutes: diffMins });
    } else if (diffMins < 1440) {
      timeString = t("clipboard.time.hoursAgo", { hours: Math.floor(diffMins / 60) });
    } else {
      timeString = t("clipboard.time.daysAgo", { days: Math.floor(diffMins / 1440) });
    }

    // Create display item
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

  // Handle delete clipboard item
  const handleDeleteItem = async (id: string) => {
    dispatch(removeClipboardItem(id));
  };

  // Handle copy to clipboard
  const handleCopyItem = async (itemId: string) => {
    try {
      console.log(`${t("clipboard.content.logs.copyItem")} ${itemId}`);
      const result = await dispatch(copyToClipboard(itemId)).unwrap();
      return result.success;
    } catch (err) {
      console.error(t("clipboard.content.logs.copyToClipboardFailed"), err);
      return false;
    }
  };

  const handleToggleFavorite = async (itemId: string, isFavorited: boolean) => {
    dispatch(toggleFavoriteItem({ id: itemId, isFavorited }));
  };

  // Clear error message
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

  // Skeleton loading state
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
          <h3 className="text-xl font-bold text-foreground mb-2">{t("clipboard.content.noClipboardItems")}</h3>
          <p className="text-muted-foreground text-center max-w-xs">
            {t("clipboard.content.emptyDescription")}
            <br />
            {t("clipboard.content.emptyHint")}
          </p>
        </div>
      )}
    </div>
  );
};

export default ClipboardContent;
