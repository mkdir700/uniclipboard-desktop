import React, { useState, useEffect } from "react";
import Header from "@/components/layout/Header";
import ClipboardContent from "@/components/clipboard/ClipboardContent";
import ActionBar from "@/components/layout/ActionBar";
import { MainLayout } from "@/layouts";
import { Filter } from "@/api/clipboardItems";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchStats } from "@/store/slices/statsSlice";

const DashboardPage: React.FC = () => {
  const [currentFilter, setCurrentFilter] = useState<Filter>(Filter.All);
  const statsState = useAppSelector((state) => state.stats);
  const dispatch = useAppDispatch();
  const handleFilterChange = (filterId: Filter) => {
    setCurrentFilter(filterId);
  };

  useEffect(() => {
    dispatch(fetchStats());
  }, [dispatch]);

  return (
    <MainLayout>
      {/* 顶部搜索栏 */}
      <Header onFilterChange={handleFilterChange} />

      {/* 剪贴板内容区域 */}
      <ClipboardContent filter={currentFilter} />

      {/* 底部快捷操作栏 */}
      <ActionBar stats={statsState.stats} />
    </MainLayout>
  );
};

export default DashboardPage;
