import React, { ReactNode } from "react";
import { Sidebar } from "@/components";

interface MainLayoutProps {
  children: ReactNode;
}

const MainLayout: React.FC<MainLayoutProps> = ({ children }) => {
  return (
    <div className="h-screen flex overflow-hidden bg-background text-foreground transition-colors duration-200">
      {/* 侧边栏导航 */}
      <Sidebar />

      {/* 主内容区域 */}
      <main className="flex-1 flex flex-col overflow-hidden relative">
        {/* Window Drag Region */}
        <div 
          data-tauri-drag-region 
          className="h-8 w-full shrink-0 absolute top-0 left-0 z-50 hover:bg-transparent"
        />
        {children}
      </main>
    </div>
  );
};

export default MainLayout;
