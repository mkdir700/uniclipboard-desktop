import React, { useState, useRef, useEffect } from "react";

import { SettingContentLayout } from "@/layouts";
import SyncSection from "@/components/setting/SyncSection";
import SecuritySection from "@/components/setting/SecuritySection";
import NetworkSection from "@/components/setting/NetworkSection";
import StorageSection from "@/components/setting/StorageSection";
import AboutSection from "@/components/setting/AboutSection";
import GeneralSection from "@/components/setting/GeneralSection";
import AppearanceSection from "@/components/setting/AppearanceSection";
import SettingHeader, { CategoryItem } from "@/components/setting/SettingHeader";

// 集中定义所有设置类别
const SETTING_CATEGORIES: CategoryItem[] = [
  { id: "general", name: "通用设置" },
  { id: "appearance", name: "外观设置" },
  { id: "sync", name: "同步设置" },
  { id: "security", name: "安全与隐私" },
  { id: "network", name: "网络设置" },
  { id: "storage", name: "存储管理" },
  { id: "about", name: "关于" },
];

const SettingsPage: React.FC = () => {
  const [activeCategory, setActiveCategory] = useState("general");
  const scrollContainerRef = useRef<HTMLDivElement>(null);

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
            <SettingContentLayout title="通用设置">
              <GeneralSection />
            </SettingContentLayout>
          </div>

          <div ref={sectionRefs.appearance} id="appearance-section" className="scroll-mt-32">
            <SettingContentLayout title="外观设置">
              <AppearanceSection />
            </SettingContentLayout>
          </div>

          <div ref={sectionRefs.sync} id="sync-section" className="scroll-mt-32">
            <SettingContentLayout title="同步设置">
              <SyncSection />
            </SettingContentLayout>
          </div>

          <div ref={sectionRefs.security} id="security-section" className="scroll-mt-32">
            <SettingContentLayout title="安全与隐私设置">
              <SecuritySection />
            </SettingContentLayout>
          </div>

          <div ref={sectionRefs.network} id="network-section" className="scroll-mt-32">
             <SettingContentLayout title="网络设置">
              <NetworkSection />
            </SettingContentLayout>
          </div>

          <div ref={sectionRefs.storage} id="storage-section" className="scroll-mt-32">
            <SettingContentLayout title="存储管理">
              <StorageSection />
            </SettingContentLayout>
          </div>

          <div ref={sectionRefs.about} id="about-section" className="scroll-mt-32">
            <SettingContentLayout title="关于">
              <AboutSection />
            </SettingContentLayout>
          </div>
        </div>
      </div>

  );
};

export default SettingsPage;
