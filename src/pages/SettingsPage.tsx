import React, { useState, useRef, useEffect } from "react";
import { SettingsLayout } from "../layouts";
import { SettingContentLayout } from "../layouts";
import SyncSection from "../components/setting/SyncSection";
import SecuritySection from "../components/setting/SecuritySection";
import NetworkSection from "../components/setting/NetworkSection";
import StorageSection from "../components/setting/StorageSection";
import AboutSection from "../components/setting/AboutSection";
import { CategoryItem } from "../components/setting/SettingHeader";

// 集中定义所有设置类别
const SETTING_CATEGORIES: CategoryItem[] = [
  { id: "sync", name: "同步设置" },
  { id: "security", name: "安全与隐私" },
  { id: "network", name: "网络设置" },
  { id: "storage", name: "存储管理" },
  { id: "about", name: "关于" },
];

const SettingsPage: React.FC = () => {
  const [activeCategory, setActiveCategory] = useState("sync");
  const [scrolling, setScrolling] = useState(false);

  // 创建对各个section的引用
  const sectionRefs = {
    sync: useRef<HTMLDivElement>(null),
    security: useRef<HTMLDivElement>(null),
    network: useRef<HTMLDivElement>(null),
    storage: useRef<HTMLDivElement>(null),
    about: useRef<HTMLDivElement>(null),
  };

  // 处理类别点击事件
  const handleCategoryClick = (category: string) => {
    setScrolling(true); // 标记为程序触发的滚动
    setActiveCategory(category);

    // 滚动到对应的section
    const sectionRef = sectionRefs[category as keyof typeof sectionRefs];
    if (sectionRef && sectionRef.current) {
      sectionRef.current.scrollIntoView({
        behavior: "smooth",
        block: "start",
      });
    }

    // 滚动完成后重置标志
    setTimeout(() => {
      setScrolling(false);
    }, 500);
  };

  // 使用 Intersection Observer 监听各个部分的可见性
  useEffect(() => {
    if (scrolling) return; // 如果是程序触发的滚动，不处理

    const observerOptions = {
      root: null, // 使用视口作为根
      rootMargin: "-10% 0px -80% 0px", // 顶部偏移10%，底部偏移80%，使顶部附近的元素被认为是可见的
      threshold: 0, // 只要有一点进入视口就触发
    };

    const observerCallback: IntersectionObserverCallback = (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          const sectionId = entry.target.id.replace("-section", "");
          if (SETTING_CATEGORIES.some((cat) => cat.id === sectionId)) {
            setActiveCategory(sectionId);
          }
        }
      });
    };

    const observer = new IntersectionObserver(
      observerCallback,
      observerOptions
    );

    // 观察所有部分
    Object.values(sectionRefs).forEach((ref) => {
      if (ref.current) {
        observer.observe(ref.current);
      }
    });

    return () => {
      observer.disconnect();
    };
  }, [scrolling]);

  return (
    <SettingsLayout
      onCategoryClick={handleCategoryClick}
      activeCategory={activeCategory}
      categories={SETTING_CATEGORIES}
    >
      <div ref={sectionRefs.sync} id="sync-section">
        <SettingContentLayout title="同步设置">
          <SyncSection />
        </SettingContentLayout>
      </div>

      <div ref={sectionRefs.security} id="security-section">
        <SettingContentLayout title="安全与隐私设置">
          <SecuritySection />
        </SettingContentLayout>
      </div>

      <div ref={sectionRefs.network} id="network-section">
        <SettingContentLayout title="网络设置">
          <NetworkSection />
        </SettingContentLayout>
      </div>

      <div ref={sectionRefs.storage} id="storage-section">
        <SettingContentLayout title="存储管理">
          <StorageSection />
        </SettingContentLayout>
      </div>

      <div ref={sectionRefs.about} id="about-section">
        <SettingContentLayout title="关于">
          <AboutSection />
        </SettingContentLayout>
      </div>
    </SettingsLayout>
  );
};

export default SettingsPage;
