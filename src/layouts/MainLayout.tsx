import React, { ReactNode } from "react";
import { Sidebar } from "../components";

interface MainLayoutProps {
  children: ReactNode;
}

const MainLayout: React.FC<MainLayoutProps> = ({ children }) => {
  return (
    <div className="h-screen flex overflow-hidden">
      {/* 侧边栏导航 */}
      <Sidebar />

      {/* 主内容区域 */}
      <div className="bg-gray-900 flex-1 flex flex-col overflow-hidden">
        {children}
      </div>
    </div>
  );
};

export default MainLayout;
