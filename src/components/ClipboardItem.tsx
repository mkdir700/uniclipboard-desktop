import React, { useState } from "react";

interface ClipboardItemProps {
  type: "text" | "image" | "link" | "code" | "file";
  title: string;
  content: string;
  time: string;
  device?: string;
  imageUrl?: string;
  isDownloaded?: boolean; // 新增：标记文件是否已下载
  onDelete?: () => void; // 新增：删除回调函数
}

const ClipboardItem: React.FC<ClipboardItemProps> = ({
  type,
  title,
  content,
  time,
  device = "",
  imageUrl,
  isDownloaded = false, // 默认未下载
  onDelete, // 删除回调函数
}) => {
  const [copySuccess, setCopySuccess] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [deleteConfirm, setDeleteConfirm] = useState(false); // 添加删除确认状态

  // 根据类型返回不同的图标和背景色
  const getTypeIcon = () => {
    switch (type) {
      case "text":
        return (
          <div className="flex-shrink-0 h-8 w-8 bg-blue-500/20 rounded-md flex items-center justify-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-4 w-4 text-blue-400"
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
          </div>
        );
      case "image":
        return (
          <div className="flex-shrink-0 h-8 w-8 bg-purple-500/20 rounded-md flex items-center justify-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-4 w-4 text-purple-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
              />
            </svg>
          </div>
        );
      case "link":
        return (
          <div className="flex-shrink-0 h-8 w-8 bg-green-500/20 rounded-md flex items-center justify-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-4 w-4 text-green-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
              />
            </svg>
          </div>
        );
      case "code":
        return (
          <div className="flex-shrink-0 h-8 w-8 bg-yellow-500/20 rounded-md flex items-center justify-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-4 w-4 text-yellow-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"
              />
            </svg>
          </div>
        );
      case "file":
        return (
          <div className="flex-shrink-0 h-8 w-8 bg-blue-500/20 rounded-md flex items-center justify-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-4 w-4 text-blue-400"
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
          </div>
        );
      default:
        return null;
    }
  };

  // 获取卡片背景样式
  const getCardStyle = () => {
    return "bg-gray-800/50 bg-opacity-60 backdrop-blur-sm border border-gray-700/40";
  };

  // 复制内容到剪贴板
  const handleCopy = () => {
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
  const performCopy = () => {
    navigator.clipboard
      .writeText(content)
      .then(() => {
        setCopySuccess(true);
        setTimeout(() => setCopySuccess(false), 2000);
      })
      .catch((err) => {
        console.error("复制失败:", err);
      });
  };

  // 处理删除操作
  const handleDelete = () => {
    if (deleteConfirm) {
      // 已经确认，执行删除操作
      console.log("删除项目:", title);
      onDelete && onDelete(); // 调用删除回调函数
    } else {
      // 首次点击，设置确认状态
      setDeleteConfirm(true);

      // 2秒后自动重置确认状态
      setTimeout(() => {
        setDeleteConfirm(false);
      }, 2000);
    }
  };

  // 渲染操作按钮
  const renderActionButtons = () => {
    return (
      <div className="flex items-center space-x-2 opacity-0 group-hover:opacity-100 transition-opacity">
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
        <button className="p-1 rounded-full hover:bg-gray-700/50">
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
              d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"
            />
          </svg>
        </button>
        <button className="p-1 rounded-full hover:bg-gray-700/50">
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
              d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"
            />
          </svg>
        </button>
        <button
          className="p-1 rounded-full hover:bg-gray-700/50"
          onClick={handleDelete}
          title={deleteConfirm ? "再次点击确认删除" : "删除"}
        >
          {deleteConfirm ? (
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
    switch (type) {
      case "text":
        return (
          <div className="mt-2 text-sm text-gray-300 line-clamp-3">
            {content}
          </div>
        );
      case "image":
        return (
          <div className="mt-2">
            <img
              src={
                imageUrl ||
                "https://images.unsplash.com/photo-1493723843671-1d655e66ac1c?ixlib=rb-1.2.1&auto=format&fit=crop&w=1050&q=80"
              }
              className="rounded-md w-full h-36 object-cover"
              alt="风景图片"
            />
          </div>
        );
      case "link":
        return (
          <div className="mt-2 p-3 bg-gray-800/50 rounded border border-gray-700/30">
            <div className="flex items-center">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-5 w-5 text-blue-400 mr-2"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              <span className="text-sm text-blue-400 hover:underline cursor-pointer truncate">
                {content}
              </span>
            </div>
            <p className="text-xs text-gray-400 mt-1">{title}</p>
          </div>
        );
      case "code":
        return (
          <div className="mt-2 bg-gray-800/70 rounded p-3 font-mono text-xs text-gray-300 overflow-x-auto">
            <pre>{content}</pre>
          </div>
        );
      case "file":
        return (
          <div className="mt-2 p-3 bg-gray-800/50 rounded border border-gray-700/30">
            <div className="flex items-center">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-5 w-5 text-blue-400 mr-2"
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
              <span className="text-sm text-gray-200 truncate">{title}</span>
            </div>
            <p className="text-xs text-gray-400 mt-1">{content}</p>
          </div>
        );
      default:
        return null;
    }
  };

  const deviceInfo = device ? `来自 ${device} · ` : "";

  return (
    <div
      className={`${getCardStyle()} rounded-lg overflow-hidden hover:ring-1 hover:ring-violet-400/40 transition duration-150 group`}
    >
      <div className="px-4 py-3">
        <div className="flex items-start justify-between">
          <div className="flex items-center space-x-2">
            {getTypeIcon()}
            <div>
              <p className="text-sm font-medium text-white">{title}</p>
              <p className="text-xs text-gray-400">
                {deviceInfo}
                {time}
              </p>
            </div>
          </div>
          {renderActionButtons()}
        </div>
        {renderContent()}
      </div>
    </div>
  );
};

export default ClipboardItem;
