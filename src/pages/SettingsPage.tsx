import React, { useState } from "react";

import { SettingContentLayout } from "@/layouts";
import SyncSection from "@/components/setting/SyncSection";
import SecuritySection from "@/components/setting/SecuritySection";
import NetworkSection from "@/components/setting/NetworkSection";
import StorageSection from "@/components/setting/StorageSection";
import AboutSection from "@/components/setting/AboutSection";
import GeneralSection from "@/components/setting/GeneralSection";
import AppearanceSection from "@/components/setting/AppearanceSection";
import SettingsSidebar from "@/components/setting/SettingsSidebar";
import { SidebarProvider, SidebarInset } from "@/components/ui/sidebar";
import { ScrollArea } from "@/components/ui/scroll-area";

const SettingsPage: React.FC = () => {
  const [activeCategory, setActiveCategory] = useState("general");

  // 处理类别点击事件
  const handleCategoryClick = (category: string) => {
    setActiveCategory(category);
  };

  // 根据选中的分类渲染对应的内容
  const renderActiveSection = () => {
    switch (activeCategory) {
      case "general":
        return (
          <SettingContentLayout>
            <GeneralSection />
          </SettingContentLayout>
        );
      case "appearance":
        return (
          <SettingContentLayout>
            <AppearanceSection />
          </SettingContentLayout>
        );
      case "sync":
        return (
          <SettingContentLayout>
            <SyncSection />
          </SettingContentLayout>
        );
      case "security":
        return (
          <SettingContentLayout>
            <SecuritySection />
          </SettingContentLayout>
        );
      case "network":
        return (
          <SettingContentLayout>
            <NetworkSection />
          </SettingContentLayout>
        );
      case "storage":
        return (
          <SettingContentLayout>
            <StorageSection />
          </SettingContentLayout>
        );
      case "about":
        return (
          <SettingContentLayout>
            <AboutSection />
          </SettingContentLayout>
        );
      default:
        return null;
    }
  };

  return (
    <SidebarProvider
      style={
        {
          "--sidebar-width": "20rem",
        } as React.CSSProperties
      }
    >
      <SettingsSidebar
        activeCategory={activeCategory}
        onCategoryChange={handleCategoryClick}
      />
      <SidebarInset>
        <ScrollArea className="flex-1">
          <div className="flex-1 p-8">
            {renderActiveSection()}
          </div>
        </ScrollArea>
      </SidebarInset>
    </SidebarProvider>
  );
};

export default SettingsPage;
