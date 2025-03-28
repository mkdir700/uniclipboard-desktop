import React, { useState } from "react";
import Header from "@/components/layout/Header";
import ClipboardContent from "@/components/clipboard/ClipboardContent";
import ActionBar from "@/components/layout/ActionBar";
import { MainLayout } from "@/layouts";
import { Filter } from "@/api/clipboardItems";
const DashboardPage: React.FC = () => {
  const [currentFilter, setCurrentFilter] = useState<Filter>(Filter.All);

  const handleFilterChange = (filterId: Filter) => {
    setCurrentFilter(filterId);
    // TODO: 根据筛选器更新剪贴板内容
  };

  return (
    <MainLayout>
      {/* 顶部搜索栏 */}
      <Header onFilterChange={handleFilterChange} />

      {/* 剪贴板内容区域 */}
      <ClipboardContent filter={currentFilter} />

      {/* 底部快捷操作栏 */}
      <ActionBar />
    </MainLayout>
  );
};

export default DashboardPage;
