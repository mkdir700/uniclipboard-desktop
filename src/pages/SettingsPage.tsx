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
import SettingsPageHeader from "@/components/setting/SettingsPageHeader";
import { SidebarProvider, SidebarInset } from "@/components/ui/sidebar";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useTranslation } from "react-i18next";

const SettingsPage: React.FC = () => {
  const { t } = useTranslation();
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
          <SettingContentLayout title={t("settings.sections.general.title")}>
            <GeneralSection />
          </SettingContentLayout>
        );
      case "appearance":
        return (
          <SettingContentLayout title={t("settings.sections.appearance.title")}>
            <AppearanceSection />
          </SettingContentLayout>
        );
      case "sync":
        return (
          <SettingContentLayout title={t("settings.sections.sync.title")}>
            <SyncSection />
          </SettingContentLayout>
        );
      case "security":
        return (
          <SettingContentLayout title={t("settings.sections.security.title")}>
            <SecuritySection />
          </SettingContentLayout>
        );
      case "network":
        return (
          <SettingContentLayout title={t("settings.sections.network.title")}>
            <NetworkSection />
          </SettingContentLayout>
        );
      case "storage":
        return (
          <SettingContentLayout title={t("settings.sections.storage.title")}>
            <StorageSection />
          </SettingContentLayout>
        );
      case "about":
        return (
          <SettingContentLayout title={t("settings.sections.about.title")}>
            <AboutSection />
          </SettingContentLayout>
        );
      default:
        return null;
    }
  };

  return (
    <SidebarProvider>
      <SettingsSidebar
        activeCategory={activeCategory}
        onCategoryChange={handleCategoryClick}
      />
      <SidebarInset>
        <SettingsPageHeader />
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
