import React from "react";
import ClipboardItem from "./ClipboardItem";

const ClipboardContent: React.FC = () => {
  // 模拟剪贴板数据
  const clipboardItems = [
    {
      id: 1,
      type: "text" as const,
      title: "会议笔记",
      content: "下周一上午10点产品评审会议，准备第三季度功能规划演示文稿。",
      time: "10分钟前",
      device: "MacBook",
    },
    {
      id: 2,
      type: "link" as const,
      title: "GitHub 仓库",
      content: "https://github.com/tauri-apps/tauri",
      time: "30分钟前",
      device: "iPhone",
    },
    {
      id: 3,
      type: "code" as const,
      title: "React 动画效果",
      content: `function animateElement(selector) {
  const element = document.querySelector(selector);
  if (!element) return;
  
  element.classList.add('animate-pulse');
  setTimeout(() => {
    element.classList.remove('animate-pulse');
  }, 1000);
}`,
      time: "1小时前",
      device: "Chrome",
    },
    {
      id: 4,
      type: "file" as const,
      title: "项目预算.xlsx",
      content: "2.4 MB · Excel 文件",
      time: "3小时前",
      device: "MacBook",
    },
    {
      id: 5,
      type: "image" as const,
      title: "产品设计稿",
      content: "设计图片",
      imageUrl:
        "https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?ixlib=rb-1.2.1&auto=format&fit=crop&w=1000&q=80",
      time: "昨天",
      device: "Figma",
    },
  ];

  return (
    <div className="flex-1 overflow-hidden">
      <div className="h-full overflow-y-auto hide-scrollbar px-4 py-4">
        <div className="space-y-4">
          {/* 今天 */}
          <div>
            <div className="sticky top-0 z-10 flex items-center pt-2 pb-3">
              <h3 className="bg-gray-800 text-sm font-semibold text-white px-3 py-1 rounded-full">
                今天
              </h3>
              <div className="flex-grow ml-3 border-t border-gray-800/50"></div>
            </div>

            <div className="space-y-3">
              {clipboardItems.slice(0, 3).map((item) => (
                <ClipboardItem
                  key={item.id}
                  type={item.type}
                  title={item.title}
                  content={item.content}
                  time={item.time}
                  device={item.device}
                  imageUrl={item.type === "image" ? item.imageUrl : undefined}
                />
              ))}
            </div>
          </div>

          {/* 昨天 */}
          <div className="mt-6">
            <div className="sticky top-0 z-10 flex items-center pt-6 pb-3">
              <h3 className="bg-gray-800 text-sm font-semibold text-white px-3 py-1 rounded-full">
                昨天
              </h3>
              <div className="flex-grow ml-3 border-t border-gray-800/50"></div>
            </div>

            <div className="space-y-3">
              {clipboardItems.slice(3).map((item) => (
                <ClipboardItem
                  key={item.id}
                  type={item.type}
                  title={item.title}
                  content={item.content}
                  time={item.time}
                  device={item.device}
                  imageUrl={item.type === "image" ? item.imageUrl : undefined}
                />
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ClipboardContent;
