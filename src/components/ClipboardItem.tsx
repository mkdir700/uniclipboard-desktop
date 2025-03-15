import React from "react";

interface ClipboardItemProps {
  type: "text" | "image" | "link" | "code" | "file";
  title: string;
  content: string;
  time: string;
  device?: string;
  imageUrl?: string;
}

const ClipboardItem: React.FC<ClipboardItemProps> = ({
  type,
  title,
  content,
  time,
  device = "",
  imageUrl,
}) => {
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
    if (type === "text") {
      return "bg-gray-900 border border-gray-800";
    }
    return "bg-gray-700 bg-opacity-60 backdrop-blur-sm border border-gray-700/40";
  };

  // 渲染操作按钮
  const renderActionButtons = () => {
    return (
      <div className="flex items-center space-x-2 opacity-0 group-hover:opacity-100 transition-opacity">
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
              d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
            />
          </svg>
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
              src={imageUrl || "https://images.unsplash.com/photo-1493723843671-1d655e66ac1c?ixlib=rb-1.2.1&auto=format&fit=crop&w=1050&q=80"}
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
    <div className={`${getCardStyle()} rounded-lg overflow-hidden hover:ring-1 hover:ring-violet-400/40 transition duration-150 group`}>
      <div className="px-4 py-3">
        <div className="flex items-start justify-between">
          <div className="flex items-center space-x-2">
            {getTypeIcon()}
            <div>
              <p className="text-sm font-medium text-white">{title}</p>
              <p className="text-xs text-gray-400">{deviceInfo}{time}</p>
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
