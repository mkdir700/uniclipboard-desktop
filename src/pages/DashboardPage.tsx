import React from "react";
import Header from "../components/Header";
import ClipboardContent from "../components/ClipboardContent";
import ActionBar from "../components/ActionBar";
import { MainLayout } from "../layouts";

const DashboardPage: React.FC = () => {
  return (
    <MainLayout>
      {/* 顶部搜索栏 */}
      <Header />

      {/* 剪贴板内容区域 */}
      <ClipboardContent />

      {/* 底部快捷操作栏 */}
      <ActionBar />
    </MainLayout>
  );
};

export default DashboardPage;
