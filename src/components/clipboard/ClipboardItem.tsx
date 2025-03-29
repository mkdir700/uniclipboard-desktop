import React, { useState, useEffect, useRef } from "react";
import { formatFileSize } from "@/utils";

interface ClipboardItemProps {
  type: "text" | "image" | "link" | "code" | "file";
  content: string;
  time: string;
  device?: string;
  imageUrl?: string;
  isDownloaded?: boolean;
  isFavorited?: boolean;
  onDelete?: () => void;
  onCopy?: () => Promise<boolean>;
  toggleFavorite?: (isFavorited: boolean) => void;
  fileSize?: number;
}

const ClipboardItem: React.FC<ClipboardItemProps> = ({
  type,
  content,
  time,
  device = "",
  imageUrl,
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
  const [downloadProgress, setDownloadProgress] = useState(0);
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

  // 获取卡片样式
  const getCardStyle = () => {
    return "bg-gray-800/60";
  };

  // 计算内容字符数
  const getCharCount = () => {
    return content.length;
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
        // 否则使用浏览器API
        await navigator.clipboard.writeText(content);
        setCopySuccess(true);
        setTimeout(() => setCopySuccess(false), 2000);
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

  // 渲染操作按钮
  const renderActionButtons = () => {
    return (
      <div className="flex items-center space-x-1">
        <button
          className="p-1 rounded-full hover:bg-gray-700/50 relative"
          onClick={handleCopy}
          title="复制到剪贴板"
          disabled={downloading}
        >
          {copySuccess ? (
            // 成功图标 - 绿色对勾
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-4 w-4 text-green-500"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fillRule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                clipRule="evenodd"
              />
            </svg>
          ) : downloading ? (
            // 圆形进度条
            <div className="relative h-4 w-4">
              <svg className="h-4 w-4" viewBox="0 0 36 36">
                {/* 背景圆 */}
                <circle
                  cx="18"
                  cy="18"
                  r="16"
                  fill="none"
                  stroke="#4B5563"
                  strokeWidth="2"
                  strokeOpacity="0.3"
                />
                {/* 进度圆 - 使用strokeDasharray和strokeDashoffset实现进度效果 */}
                <circle
                  cx="18"
                  cy="18"
                  r="16"
                  fill="none"
                  stroke="#3B82F6"
                  strokeWidth="2"
                  strokeLinecap="round"
                  transform="rotate(-90 18 18)"
                  strokeDasharray="100"
                  strokeDashoffset={100 - downloadProgress}
                  style={{
                    transition: "stroke-dashoffset 0.2s ease",
                  }}
                />
              </svg>
            </div>
          ) : (
            // 默认复制图标
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-4 w-4 text-gray-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
              />
            </svg>
          )}
        </button>

        {/* 收藏按钮 */}
        <button
          className="p-1 rounded-full hover:bg-gray-700/50"
          onClick={handleFavoriteClick}
          title={favorited ? "取消收藏" : "收藏"}
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className={`h-4 w-4 ${
              favorited ? "text-yellow-400 fill-yellow-400" : "text-gray-400"
            }`}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"
            />
          </svg>
        </button>

        {/* 删除按钮 */}
        <button
          className="p-1 rounded-full hover:bg-gray-700/50"
          onClick={handleDeleteClick}
          title={deleteConfirmation ? "再次点击确认删除" : "删除"}
        >
          {deleteConfirmation ? (
            // 确认删除状态 - 红色X图标
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-4 w-4 text-red-500"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fillRule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                clipRule="evenodd"
              />
            </svg>
          ) : (
            // 默认删除图标
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-4 w-4 text-gray-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
              />
            </svg>
          )}
        </button>
      </div>
    );
  };

  // 渲染内容
  const renderContent = () => {
    const contentNeedsExpand =
      type === "image"
        ? needsImageExpand(imageHeight)
        : needsExpandCollapse(content, type);

    // 渐变蒙版组件
    const renderGradientMask = () => {
      if (isExpanded || !contentNeedsExpand) return null;

      return (
        <div className="absolute bottom-0 left-0 right-0 h-12 bg-gradient-to-t from-gray-800/90 to-transparent pointer-events-none">
          {/* 渐变蒙版，提示内容被截断 */}
        </div>
      );
    };

    switch (type) {
      case "text":
        return (
          <div className="p-2 bg-gray-800/50 rounded border border-gray-700/30 font-mono text-xs text-gray-300 overflow-x-auto relative">
            <pre
              className={`whitespace-pre-wrap ${
                isExpanded ? "" : "line-clamp-2"
              }`}
            >
              {content}
            </pre>
            {renderGradientMask()}
          </div>
        );
      case "image":
        const imageNeedsExpand = needsImageExpand(imageHeight);
        return (
          <div
            className={`relative overflow-hidden ${
              isExpanded || !imageNeedsExpand ? "" : "max-h-[300px]"
            }`}
          >
            <img
              ref={imageRef}
              src={
                imageUrl ||
                "https://images.unsplash.com/photo-1493723843671-1d655e66ac1c?ixlib=rb-1.2.1&auto=format&fit=crop&w=1050&q=80"
              }
              className="w-full object-contain rounded-md"
              alt="图片"
              onLoad={handleImageLoad}
            />
            {imageNeedsExpand && !isExpanded && (
              <div className="absolute bottom-0 left-0 right-0 h-16 bg-gradient-to-t from-gray-800/90 to-transparent">
                {/* 渐变蒙版，提示图片被截断 */}
              </div>
            )}
          </div>
        );
      case "link":
        return (
          <div className="p-2 bg-gray-800/50 rounded border border-gray-700/30 font-mono text-xs text-gray-300 overflow-x-auto relative">
            <div className={`${isExpanded ? "" : "line-clamp-2"} break-all`}>
              <a
                href={content}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-400 hover:underline"
                onClick={(e) => {
                  if (contentNeedsExpand && !isExpanded) {
                    e.preventDefault();
                    setIsExpanded(true);
                  }
                }}
              >
                {content}
              </a>
            </div>
            {renderGradientMask()}
          </div>
        );
      case "code":
        return (
          <div className="p-2 bg-gray-800/50 rounded border border-gray-700/30 font-mono text-xs text-gray-300 overflow-x-auto relative">
            <pre
              className={`whitespace-pre-wrap ${
                isExpanded ? "" : "line-clamp-2"
              }`}
            >
              {content}
            </pre>
            {renderGradientMask()}
          </div>
        );
      case "file":
        return (
          <div className="p-2 bg-gray-800/50 rounded border border-gray-700/30 font-mono text-xs text-gray-300 overflow-x-auto relative">
            <div className="flex items-center">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-4 w-4 text-gray-400 mr-2 flex-shrink-0"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                />
              </svg>
              <pre
                className={`whitespace-pre-wrap ${
                  isExpanded ? "" : "line-clamp-2"
                }`}
              >
                {content}
              </pre>
            </div>
            {renderGradientMask()}
          </div>
        );
      default:
        return null;
    }
  };

  const deviceInfo = device ? `${device} · ` : "";
  const shouldUseExpand =
    type === "image"
      ? needsImageExpand(imageHeight)
      : needsExpandCollapse(content, type);
  const sizeInfo = getSizeInfo();

  return (
    <div
      className={`${getCardStyle()} rounded-lg overflow-hidden hover:ring-1 hover:ring-violet-400/40 transition duration-150 group mb-3 relative`}
    >
      <div className="p-3">
        {/* 右上角操作按钮 */}
        <div className="absolute top-2 right-2 z-10">
          <div className="flex items-center space-x-1 bg-gray-800/80 rounded-md p-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
            {renderActionButtons()}
          </div>
        </div>

        <div className="flex items-start">
          <div className="flex-1">
            {renderContent()}
            <div className="flex justify-between items-center mt-2">
              <div className="text-xs text-gray-400">{time}</div>

              {shouldUseExpand && (
                <button
                  onClick={() => setIsExpanded(!isExpanded)}
                  className="text-xs text-gray-400 hover:text-gray-300 flex items-center mx-2"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-4 w-4 mr-1"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth="2"
                      d={isExpanded ? "M5 15l7-7 7 7" : "M19 9l-7 7-7-7"}
                    />
                  </svg>
                  {isExpanded ? "收起" : "展开"}
                </button>
              )}

              <div className="text-xs text-gray-400">{sizeInfo}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ClipboardItem;
