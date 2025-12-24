import React, { useState, useEffect, useRef } from "react";
import {
  Copy,
  Check,
  Star,
  Trash2,
  ChevronDown,
  ChevronUp,
  FileText,
  Image as ImageIcon,
  Link as LinkIcon,
  File,
  Code,
} from "lucide-react";
import { formatFileSize } from "@/utils";
import {
  ClipboardTextItem,
  ClipboardImageItem,
  ClipboardLinkItem,
  ClipboardCodeItem,
  ClipboardFileItem,
} from "@/api/clipboardItems";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface ClipboardItemProps {
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
  onDelete?: () => void;
  onCopy?: () => Promise<boolean>;
  toggleFavorite?: (isFavorited: boolean) => void;
  fileSize?: number;
}

const ClipboardItem: React.FC<ClipboardItemProps> = ({
  type,
  time,
  content,
  isDownloaded = false,
  isFavorited = false,
  onDelete,
  onCopy,
  toggleFavorite,
  fileSize,
}) => {
  const [copySuccess, setCopySuccess] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [favorited, setFavorited] = useState(isFavorited);
  const [downloadProgress, setDownloadProgress] = useState(0); // eslint-disable-line @typescript-eslint/no-unused-vars
  const [deleteConfirmation, setDeleteConfirmation] = useState(false);
  const [deleteTimer, setDeleteTimer] = useState<ReturnType<
    typeof setTimeout
  > | null>(null);
  const [isExpanded, setIsExpanded] = useState(false);
  const [imageHeight, setImageHeight] = useState<number | null>(null);
  const imageRef = useRef<HTMLImageElement>(null);

  // 组件卸载时清除计时器
  useEffect(() => {
    return () => {
      if (deleteTimer) {
        clearTimeout(deleteTimer);
      }
    };
  }, [deleteTimer]);

  // 计算内容字符数
  const getCharCount = () => {
    if (type === "text") {
      return (content as ClipboardTextItem).display_text.length;
    } else if (type === "link") {
      return (content as ClipboardLinkItem).url.length;
    } else if (type === "code") {
      return (content as ClipboardCodeItem).code.length;
    } else if (type === "file") {
      return (content as ClipboardFileItem).file_names.length;
    } else if (type === "image") {
      return (content as ClipboardImageItem).thumbnail.length;
    } else if (type === "unknown") {
      return 0;
    }
    return 0;
  };

  // 获取内容大小信息
  const getSizeInfo = (): string => {
    if (type === "text" || type === "link" || type === "code") {
      return `${getCharCount()} 字符`;
    } else if (type === "image" || type === "file") {
      return formatFileSize(fileSize);
    }
    return "";
  };

  // 复制内容到剪贴板
  const handleCopy = async () => {
    // 如果是文件类型且未下载，先模拟下载过程
    if (type === "file" && !isDownloaded) {
      setDownloading(true);
      setDownloadProgress(0);

      // 模拟下载进度
      const downloadInterval = setInterval(() => {
        setDownloadProgress((prev) => {
          const newProgress = prev + 10;
          if (newProgress >= 100) {
            clearInterval(downloadInterval);
            setDownloading(false);
            // 下载完成后复制
            performCopy();
            return 100;
          }
          return newProgress;
        });
      }, 200);
    } else {
      // 文本、图片或已下载的文件直接复制
      performCopy();
    }
  };

  // 执行复制操作
  const performCopy = async () => {
    try {
      // 如果提供了onCopy回调则使用它
      if (onCopy) {
        const success = await onCopy();
        if (success) {
          setCopySuccess(true);
          setTimeout(() => setCopySuccess(false), 2000);
        }
      } else {
        console.error("没有提供onCopy回调");
      }
    } catch (err) {
      console.error("复制失败:", err);
    }
  };

  // 处理收藏操作
  const handleFavoriteClick = () => {
    const newFavorited = !favorited;
    console.log("handleFavoriteClick", newFavorited);
    setFavorited(newFavorited);
    toggleFavorite && toggleFavorite(newFavorited);
  };

  // 处理删除操作
  const handleDeleteClick = () => {
    if (deleteConfirmation) {
      if (deleteTimer) {
        clearTimeout(deleteTimer);
        setDeleteTimer(null);
      }
      onDelete && onDelete(); // 调用删除回调函数
      setDeleteConfirmation(false);
    } else {
      // 首次点击，设置确认状态
      setDeleteConfirmation(true);

      // 2秒后自动重置确认状态
      const timer = setTimeout(() => {
        setDeleteConfirmation(false);
      }, 2000);
      setDeleteTimer(timer);
    }
  };

  // 图片加载完成后检查高度
  const handleImageLoad = () => {
    if (imageRef.current) {
      setImageHeight(imageRef.current.naturalHeight);
    }
  };

  // 图片最大高度阈值
  const IMAGE_HEIGHT_THRESHOLD = 300;

  // 检查图片是否需要展开/折叠
  const needsImageExpand = (height: number | null): boolean => {
    return height !== null && height > IMAGE_HEIGHT_THRESHOLD;
  };

  // 检查内容是否需要展开/收起功能
  const needsExpandCollapse = (content: string, type: string): boolean => {
    if (type === "image") {
      return needsImageExpand(imageHeight);
    }

    // 链接类型特殊处理：基于字符长度判断
    if (type === "link") {
      // 链接如果超过100个字符，认为需要展开/收起
      return content.length > 100;
    }

    // 计算内容的行数
    const lines = content.split("\n");
    // 如果超过两行，就需要展开/收起功能
    return lines.length > 2;
  };

  // 获取展示的文本内容
  const getContentText = (): string => {
    switch (type) {
      case "text":
        return (content as ClipboardTextItem).display_text;
      case "link":
        return (content as ClipboardLinkItem).url;
      case "code":
        return (content as ClipboardCodeItem).code;
      case "file":
        return (content as ClipboardFileItem).file_names.join(", ");
      case "image":
        return (content as ClipboardImageItem).thumbnail;
      case "unknown":
      default:
        return "未知内容";
    }
  };

  const shouldUseExpand =
    type === "image"
      ? needsImageExpand(imageHeight)
      : needsExpandCollapse(getContentText(), type);
  const sizeInfo = getSizeInfo();
  const contentNeedsExpand =
    type === "image"
      ? needsImageExpand(imageHeight)
      : needsExpandCollapse(getContentText(), type);

  // 渲染内容
  const renderContent = () => {
    // 渐变蒙版组件
    const renderGradientMask = () => {
      if (isExpanded || !contentNeedsExpand) return null;

      return (
        <div className="absolute bottom-0 left-0 right-0 h-12 bg-gradient-to-t from-card/90 to-transparent pointer-events-none" />
      );
    };

    const contentClassName = cn(
      "p-3 bg-muted/50 rounded-lg border border-border font-mono text-sm overflow-x-auto relative"
    );

    const textClassName = cn(
      "whitespace-pre-wrap",
      !isExpanded && "line-clamp-4"
    );

    switch (type) {
      case "text":
        return (
          <div className={contentClassName}>
            <pre className={textClassName}>{getContentText()}</pre>
            {renderGradientMask()}
          </div>
        );
      case "image":
        const imageNeedsExpand = needsImageExpand(imageHeight);
        const imageItem = content as ClipboardImageItem;
        return (
          <div
            className={cn(
              "relative overflow-hidden rounded-lg bg-muted/30",
              !isExpanded && imageNeedsExpand && "max-h-[200px]"
            )}
          >
            <img
              ref={imageRef}
              src={imageItem.thumbnail}
              className="w-full object-contain"
              alt="图片"
              onLoad={handleImageLoad}
            />
            {imageNeedsExpand && !isExpanded && (
              <div className="absolute bottom-0 left-0 right-0 h-16 bg-gradient-to-t from-card/90 to-transparent" />
            )}
          </div>
        );
      case "link":
        const linkUrl = (content as ClipboardLinkItem).url;
        return (
          <div className={contentClassName}>
            <div className={cn("break-all", !isExpanded && "line-clamp-4")}>
              <a
                href={linkUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 dark:text-blue-400 hover:underline"
                onClick={(e) => {
                  if (contentNeedsExpand && !isExpanded) {
                    e.preventDefault();
                    setIsExpanded(true);
                  }
                }}
              >
                {linkUrl}
              </a>
            </div>
            {renderGradientMask()}
          </div>
        );
      case "code":
        const codeContent = (content as ClipboardCodeItem).code;
        return (
          <div className={contentClassName}>
            <pre className={textClassName}>{codeContent}</pre>
            {renderGradientMask()}
          </div>
        );
      case "file":
        const fileNames = (content as ClipboardFileItem).file_names.join(", ");
        return (
          <div className={contentClassName}>
            <div className="flex items-center gap-2">
              <File className="h-4 w-4 text-muted-foreground flex-shrink-0" />
              <pre className={textClassName}>{fileNames}</pre>
            </div>
            {renderGradientMask()}
          </div>
        );
      default:
        return null;
    }
  };

  // 获取类型图标
  const getTypeIcon = () => {
    switch (type) {
      case "text":
        return <FileText className="h-4 w-4" />;
      case "image":
        return <ImageIcon className="h-4 w-4" />;
      case "link":
        return <LinkIcon className="h-4 w-4" />;
      case "code":
        return <Code className="h-4 w-4" />;
      case "file":
        return <File className="h-4 w-4" />;
      default:
        return null;
    }
  };

  return (
    <Card className="bg-card rounded-2xl p-5 border border-border shadow-sm hover:shadow-md transition-shadow group flex flex-col h-56">
      <CardContent className="p-0 flex flex-col h-full">
        {/* 时间戳 */}
        <div className="text-xs text-muted-foreground font-medium mb-3">
          {time}
        </div>

        {/* 内容区域 */}
        <div className="flex-1 overflow-hidden">
          {renderContent()}
        </div>

        {/* 底部操作栏 */}
        <div className="mt-4 pt-3 border-t border-border flex justify-between items-center">
          {/* 左侧：类型图标 + 大小信息 */}
          <div className="flex items-center gap-2 text-muted-foreground">
            {getTypeIcon()}
            <span className="text-xs">{sizeInfo}</span>
          </div>

          {/* 右侧：操作按钮 */}
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 hover:text-primary transition-colors"
              onClick={handleCopy}
              disabled={downloading}
              title="复制"
            >
              {copySuccess ? (
                <Check className="h-4 w-4 text-success" />
              ) : (
                <Copy className="h-4 w-4" />
              )}
            </Button>

            <Button
              variant="ghost"
              size="icon"
              className={cn(
                "h-8 w-8 transition-colors",
                favorited
                  ? "text-amber-500 hover:text-amber-600"
                  : "hover:text-amber-500"
              )}
              onClick={handleFavoriteClick}
              title={favorited ? "取消收藏" : "收藏"}
            >
              <Star className={cn("h-4 w-4", favorited && "fill-current")} />
            </Button>

            <Button
              variant="ghost"
              size="icon"
              className={cn(
                "h-8 w-8 transition-colors",
                deleteConfirmation
                  ? "text-destructive"
                  : "hover:text-destructive"
              )}
              onClick={handleDeleteClick}
              title={deleteConfirmation ? "再次点击确认删除" : "删除"}
            >
              {deleteConfirmation ? (
                <Check className="h-4 w-4" />
              ) : (
                <Trash2 className="h-4 w-4" />
              )}
            </Button>
          </div>
        </div>

        {/* 展开/收起按钮 */}
        {shouldUseExpand && (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setIsExpanded(!isExpanded)}
            className="absolute bottom-16 left-1/2 -translate-x-1/2 h-6 px-2 text-xs text-muted-foreground hover:text-foreground"
          >
            {isExpanded ? (
              <>
                <ChevronUp className="h-3 w-3 mr-1" />
                收起
              </>
            ) : (
              <>
                <ChevronDown className="h-3 w-3 mr-1" />
                展开
              </>
            )}
          </Button>
        )}
      </CardContent>
    </Card>
  );
};

export default ClipboardItem;
