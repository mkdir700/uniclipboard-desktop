import React, { useState, useRef, useEffect, useMemo } from "react";

import { SettingContentLayout } from "@/layouts";
import SyncSection from "@/components/setting/SyncSection";
import SecuritySection from "@/components/setting/SecuritySection";
import NetworkSection from "@/components/setting/NetworkSection";
import StorageSection from "@/components/setting/StorageSection";
import AboutSection from "@/components/setting/AboutSection";
import GeneralSection from "@/components/setting/GeneralSection";
import AppearanceSection from "@/components/setting/AppearanceSection";
import SettingHeader, { CategoryItem } from "@/components/setting/SettingHeader";
import { useTranslation } from "react-i18next";

const SettingsPage: React.FC = () => {
  const { t } = useTranslation();
  const [activeCategory, setActiveCategory] = useState("general");
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  const SETTING_CATEGORIES: CategoryItem[] = useMemo(() => [
    { id: "general", name: t("settings.categories.general") },
    { id: "appearance", name: t("settings.categories.appearance") },
    { id: "sync", name: t("settings.categories.sync") },
    { id: "security", name: t("settings.categories.security") },
    { id: "network", name: t("settings.categories.network") },
    { id: "storage", name: t("settings.categories.storage") },
    { id: "about", name: t("settings.categories.about") },
  ], [t]);

  // 创建对各个section的引用
  const sectionRefs = {
    general: useRef<HTMLDivElement>(null),
    appearance: useRef<HTMLDivElement>(null),
    sync: useRef<HTMLDivElement>(null),
    security: useRef<HTMLDivElement>(null),
    network: useRef<HTMLDivElement>(null),
    storage: useRef<HTMLDivElement>(null),
    about: useRef<HTMLDivElement>(null),
  };

  // 处理类别点击事件
  const handleCategoryClick = (category: string) => {
    setActiveCategory(category);

    // 滚动到对应的section
    const sectionRef = sectionRefs[category as keyof typeof sectionRefs];
    if (sectionRef && sectionRef.current) {
      sectionRef.current.scrollIntoView({
        behavior: "smooth",
        block: "start",
      });
    }
  };

  // 监听滚动更新 activeCategory
  useEffect(() => {
    const container = scrollContainerRef.current;
    if (!container) return;

    const handleScroll = () => {
      // 找到最近顶部的 section
      let currentActive = activeCategory;
      let minDistance = Infinity;

      Object.entries(sectionRefs).forEach(([id, ref]) => {
        if (ref.current) {
          const rect = ref.current.getBoundingClientRect();
          // 我们关注那些接近顶部，或者已经在视口中的元素
          // 使用绝对值来找最近的一个
          const distance = Math.abs(rect.top - 150); // 150 offset for header
          if (distance < minDistance) {
            minDistance = distance;
            currentActive = id;
          }
        }
      });
      
      if (currentActive !== activeCategory) {
        setActiveCategory(currentActive);
      }
    };

    container.addEventListener("scroll", handleScroll);
    // Initial check
    handleScroll();
    
    return () => container.removeEventListener("scroll", handleScroll);
  }, [activeCategory]); // Add activeCategory dependency if needed, but actually we want to update it

  return (

      <div className="flex flex-col h-full relative">
        <SettingHeader
          onCategoryClick={handleCategoryClick}
          activeCategory={activeCategory}
          categories={SETTING_CATEGORIES}
        />

        <div 
          ref={scrollContainerRef}
          className="flex-1 overflow-y-auto scrollbar-thin px-8 pb-32 pt-6 scroll-smooth"
        >
          <div ref={sectionRefs.general} id="general-section" className="scroll-mt-32">
            <SettingContentLayout title={t("settings.sections.general.title")}>
              <GeneralSection />
            </SettingContentLayout>
          </div>

          <div ref={sectionRefs.appearance} id="appearance-section" className="scroll-mt-32">
            <SettingContentLayout title={t("settings.sections.appearance.title")}>
              <AppearanceSection />
            </SettingContentLayout>
          </div>

          <div ref={sectionRefs.sync} id="sync-section" className="scroll-mt-32">
            <SettingContentLayout title={t("settings.sections.sync.title")}>
              <SyncSection />
            </SettingContentLayout>
          </div>

          <div ref={sectionRefs.security} id="security-section" className="scroll-mt-32">
            <SettingContentLayout title={t("settings.sections.security.title")}>
              <SecuritySection />
            </SettingContentLayout>
          </div>

          <div ref={sectionRefs.network} id="network-section" className="scroll-mt-32">
             <SettingContentLayout title={t("settings.sections.network.title")}>
              <NetworkSection />
            </SettingContentLayout>
          </div>

          <div ref={sectionRefs.storage} id="storage-section" className="scroll-mt-32">
            <SettingContentLayout title={t("settings.sections.storage.title")}>
              <StorageSection />
            </SettingContentLayout>
          </div>

          <div ref={sectionRefs.about} id="about-section" className="scroll-mt-32">
            <SettingContentLayout title={t("settings.sections.about.title")}>
              <AboutSection />
            </SettingContentLayout>
          </div>
        </div>
      </div>

  );
};

export default SettingsPage;
