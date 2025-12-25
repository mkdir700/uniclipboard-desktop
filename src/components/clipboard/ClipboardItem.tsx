import React, { useState } from "react";
import {
  Copy,
  Check,
  Star,
  Trash2,
  ChevronDown,
  ChevronUp,
  File,
  ExternalLink,
  MoreHorizontal
} from "lucide-react";
import { formatFileSize } from "@/utils";
import {
  ClipboardTextItem,
  ClipboardImageItem,
  ClipboardLinkItem,
  ClipboardCodeItem,
  ClipboardFileItem,
} from "@/api/clipboardItems";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useTranslation } from "react-i18next";

interface ClipboardItemProps {
  index: number;
  type: "text" | "image" | "link" | "code" | "file" | "unknown";
  time: string;
  device?: string;
  content:
    | ClipboardTextItem
    | ClipboardImageItem
    | ClipboardLinkItem
    | ClipboardCodeItem
    | ClipboardFileItem
    | null;
  isDownloaded?: boolean;
  isFavorited?: boolean;
  isSelected?: boolean;
  onSelect?: () => void;
  onDelete?: () => void;
  onCopy?: () => Promise<boolean>;
  toggleFavorite?: (isFavorited: boolean) => void;
  fileSize?: number;
}

const ClipboardItem: React.FC<ClipboardItemProps> = ({
  index,
  type,
  time,
  content,
  isDownloaded = false,
  isFavorited = false,
  isSelected = false,
  onSelect,
  onDelete,
  onCopy,
  toggleFavorite,
  fileSize,
}) => {
  const { t } = useTranslation();
  const [copySuccess, setCopySuccess] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);
  const [isHovered, setIsHovered] = useState(false);

  // Calculate character count or size info
  const getSizeInfo = (): string => {
    if (!content) return "";
    switch(type) {
        case "text": return `${(content as ClipboardTextItem).display_text.length} ${t("clipboard.item.characters")}`;
        case "link": return t("clipboard.item.link");
        case "code": return `${(content as ClipboardCodeItem).code.length} ${t("clipboard.item.characters")}`;
        case "file": return formatFileSize(fileSize);
        case "image":
            // Note: Use actual dimensions if available in API, otherwise placeholder or remove
            return t("clipboard.item.image");
        default: return "";
    }
  };

  const handleCopy = async (e?: React.MouseEvent) => {
    e?.stopPropagation();
    if (type === "file" && !isDownloaded) {
      setTimeout(() => {
          performCopy();
      }, 1000);
    } else {
      performCopy();
    }
  };

  const performCopy = async () => {
    if (onCopy) {
      const success = await onCopy();
      if (success) {
        setCopySuccess(true);
        setTimeout(() => setCopySuccess(false), 2000);
      }
    }
  };

  const handleFavoriteClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    toggleFavorite?.(!isFavorited);
  };

  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onDelete?.();
  };

  const renderContent = () => {
    switch (type) {
      case "text":
        return (
          <p className={cn(
            "whitespace-pre-wrap font-mono text-sm leading-relaxed text-foreground/90 break-words",
            !isExpanded && "line-clamp-5"
          )}>
            {(content as ClipboardTextItem).display_text}
          </p>
        );
      case "image":
        return (
          <div className="flex justify-center bg-black/20 rounded-lg overflow-hidden py-4">
            <img
              src={(content as ClipboardImageItem).thumbnail}
              className={cn(
                  "w-auto object-contain rounded-md shadow-sm transition-all duration-300",
                  isExpanded ? "max-h-[500px]" : "h-32"
              )}
              alt={t("clipboard.item.altText.clipboardImage")}
              loading="lazy"
            />
          </div>
        );
      case "link":
        const url = (content as ClipboardLinkItem).url;
        return (
            <div className="flex flex-col gap-1">
                <a 
                    href={url} 
                    target="_blank" 
                    rel="noreferrer" 
                    className="text-primary font-medium hover:underline break-all text-sm leading-relaxed flex items-center gap-2"
                    onClick={(e) => e.stopPropagation()}
                >
                    <ExternalLink size={14} />
                    {url}
                </a>
            </div>
        );
      case "code":
        return (
            <div className="bg-muted/30 p-3 rounded-lg border border-border/30 overflow-hidden font-mono text-xs">
                <pre className={cn("whitespace-pre-wrap break-all text-foreground/80", !isExpanded && "line-clamp-6")}>
                    {(content as ClipboardCodeItem).code}
                </pre>
            </div>
        );
      case "file":
        const fileNames = (content as ClipboardFileItem).file_names;
        return (
          <div className="flex flex-col gap-2">
             {fileNames.map((name, i) => (
                 <div key={i} className="flex items-center gap-2 text-sm text-foreground/80">
                      <File size={16} className="text-muted-foreground" />
                      <span className="truncate">{name}</span>
                 </div>
             ))}
          </div>
        );
      default:
        return <p className="text-muted-foreground text-sm">{t("clipboard.item.unknownContent")}</p>;
    }
  };

  return (
    <div
      className={cn(
        "group relative flex flex-col border-b border-border/40 transition-colors duration-200 select-none",
        isSelected 
            ? "bg-primary/5 border-l-4 border-l-primary" 
            : "hover:bg-muted/20 border-l-4 border-l-transparent hover:border-l-primary/30"
      )}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      onClick={onSelect}
    >
        {/* Main Content Area */}
        <div className="p-4 pr-12"> 
            {renderContent()}
        </div>

        {/* Footer Area */}
        <div className="flex items-center justify-between px-4 pb-2 pt-1 text-xs text-muted-foreground/60 select-none">
            {/* Left: Time */}
            <div className="min-w-[80px]">
                {time}
            </div>

            {/* Center: Expand Button (Visible if expandable content, or always visible logic) */}
            <div 
                className="flex items-center gap-1 cursor-pointer hover:text-foreground transition-colors px-2 py-1 rounded-md hover:bg-muted/50"
                onClick={(e) => {
                    e.stopPropagation();
                    setIsExpanded(!isExpanded);
                }}
            >
                {isExpanded ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
                <span>{isExpanded ? t("clipboard.item.collapse") : t("clipboard.item.expand")}</span>
            </div>

            {/* Right: Stats & Index */}
            <div className="flex items-center gap-4 min-w-[80px] justify-end">
                <span>{getSizeInfo()}</span>
                <span className="font-mono text-muted-foreground/40">{index}</span>
            </div>
        </div>

        {/* Floating Actions (Right Side) */}
        <div className="absolute top-4 right-2 flex flex-col gap-1 z-10">
            {/* Context Menu / More */}
             <DropdownMenu>
                <DropdownMenuTrigger asChild>
                    <Button
                        size="icon"
                        variant="ghost" 
                        className="h-8 w-8 text-muted-foreground/50 hover:text-foreground hover:bg-muted/50 transition-colors"
                        onClick={(e) => e.stopPropagation()}
                    >
                         <MoreHorizontal size={16} />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-40">
                    <DropdownMenuItem onClick={handleCopy}>
                        <Copy size={14} className="mr-2" />
                        <span>{t("clipboard.item.actions.copy")}</span>
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={handleFavoriteClick}>
                        <Star size={14} className={cn("mr-2", isFavorited && "fill-current text-amber-500")} />
                        <span>{isFavorited ? t("clipboard.item.actions.unfavorite") : t("clipboard.item.actions.favorite")}</span>
                    </DropdownMenuItem>
                    <DropdownMenuItem className="text-destructive focus:text-destructive" onClick={handleDeleteClick}>
                        <Trash2 size={14} className="mr-2" />
                        <span>{t("clipboard.item.actions.delete")}</span>
                    </DropdownMenuItem>
                </DropdownMenuContent>
             </DropdownMenu>

            {/* Quick Actions (Visible on Hover) */}
            <div className={cn(
                "flex flex-col gap-1 transition-opacity duration-200",
                isHovered ? "opacity-100" : "opacity-0"
            )}>
                 <Button
                    size="icon"
                    variant="ghost"
                    className="h-8 w-8 text-muted-foreground/50 hover:text-primary hover:bg-primary/10"
                    onClick={(e) => { e.stopPropagation(); handleCopy(e); }}
                    title={t("clipboard.item.actions.copy")}
                >
                    {copySuccess ? <Check size={16} className="text-success" /> : <Copy size={16} />}
                </Button>
                 <Button
                    size="icon"
                    variant="ghost"
                    className={cn(
                        "h-8 w-8 text-muted-foreground/50 hover:text-amber-500 hover:bg-amber-500/10",
                        isFavorited && "text-amber-500 opacity-100"
                    )}
                    onClick={handleFavoriteClick}
                    title={t("clipboard.item.actions.favorite")}
                >
                    <Star size={16} className={cn(isFavorited && "fill-current")} />
                </Button>
            </div>
        </div>
    </div>
  );
};

export default ClipboardItem;
