import React, { useMemo, useState } from "react";
import { Check, Copy, Star, Trash2, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface ClipboardSelectionActionBarProps {
  selectedCount: number;
  favoriteIntent: "favorite" | "unfavorite";
  onCopy: () => Promise<boolean> | boolean;
  onToggleFavorite: () => Promise<void> | void;
  onDelete: () => Promise<void> | void;
  onClearSelection?: () => void;
}

const ClipboardSelectionActionBar: React.FC<
  ClipboardSelectionActionBarProps
> = ({
  selectedCount,
  favoriteIntent,
  onCopy,
  onToggleFavorite,
  onDelete,
  onClearSelection,
}) => {
  const [copySuccess, setCopySuccess] = useState(false);

  const favoriteTitle = useMemo(() => {
    if (favoriteIntent === "unfavorite") return "取消收藏";
    return "收藏";
  }, [favoriteIntent]);

  const handleCopyClick = async () => {
    const ok = await onCopy();
    if (ok) {
      setCopySuccess(true);
      window.setTimeout(() => setCopySuccess(false), 1500);
    }
  };

  if (selectedCount === 0) {
    return null;
  }

  return (
    <div className="absolute bottom-0 left-0 right-0 z-20 px-4 pb-4 pt-8 bg-linear-to-t from-background via-background/95 to-transparent">
      <div className="glass-strong border border-border/60 rounded-2xl shadow-lg max-w-2xl mx-auto">
        <div className="flex items-center justify-between px-4 py-3">
          {/* Left: Selection info */}
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">
                {selectedCount === 1
                  ? "已选择 1 项"
                  : `已选择 ${selectedCount} 项`}
              </span>
            </div>
          </div>

          {/* Right: Action buttons */}
          <div className="flex items-center gap-1">
            <Button
              size="sm"
              variant="ghost"
              className={cn(
                "h-9 px-3 gap-2 rounded-lg transition-all duration-200",
                "hover:bg-primary/10 hover:text-primary"
              )}
              title={selectedCount > 1 ? `复制（${selectedCount} 项）` : "复制"}
              onClick={handleCopyClick}
            >
              {copySuccess ? (
                <>
                  <Check className="h-4 w-4" />
                  <span className="text-xs">已复制</span>
                </>
              ) : (
                <>
                  <Copy className="h-4 w-4" />
                  <span className="text-xs">复制</span>
                </>
              )}
            </Button>

            <Button
              size="sm"
              variant="ghost"
              className={cn(
                "h-9 px-3 gap-2 rounded-lg transition-all duration-200",
                favoriteIntent === "unfavorite"
                  ? "text-amber-500 hover:bg-amber-500/10 hover:text-amber-500"
                  : "hover:bg-amber-500/10 hover:text-amber-500"
              )}
              title={
                selectedCount > 1
                  ? `${favoriteTitle}（${selectedCount} 项）`
                  : favoriteTitle
              }
              onClick={onToggleFavorite}
            >
              <Star
                className={cn(
                  "h-4 w-4",
                  favoriteIntent === "unfavorite" && "fill-current"
                )}
              />
              <span className="text-xs">{favoriteTitle}</span>
            </Button>

            <Button
              size="sm"
              variant="ghost"
              className="h-9 px-3 gap-2 rounded-lg text-destructive hover:bg-destructive/10 hover:text-destructive transition-all duration-200"
              title={selectedCount > 1 ? `删除（${selectedCount} 项）` : "删除"}
              onClick={onDelete}
            >
              <Trash2 className="h-4 w-4" />
              <span className="text-xs">删除</span>
            </Button>

            <div className="w-px h-6 bg-border/50 mx-1" />

            <Button
              size="sm"
              variant="ghost"
              className="h-9 w-9 p-0 rounded-lg hover:bg-muted transition-all duration-200"
              title="取消选择"
              onClick={onClearSelection}
            >
              <X className="h-4 w-4 text-muted-foreground" />
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ClipboardSelectionActionBar;
