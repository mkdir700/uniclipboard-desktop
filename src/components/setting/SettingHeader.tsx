import React from "react";
import { motion } from "framer-motion";
import { cn } from "@/lib/utils";

export interface CategoryItem {
  id: string;
  name: string;
}

interface SettingHeaderProps {
  onCategoryClick: (category: string) => void;
  activeCategory: string;
  categories: CategoryItem[];
}

const SettingHeader: React.FC<SettingHeaderProps> = ({
  onCategoryClick,
  activeCategory,
  categories,
}) => {
  return (
    <header 
      data-tauri-drag-region
      className="sticky top-0 z-50 pt-6 pb-2 px-8 transition-all duration-300"
    >
      {/* Glass Background */}
      <div
        data-tauri-drag-region
        className="absolute inset-0 bg-background/60 backdrop-blur-xl border-b border-white/5 shadow-sm"
      />

      <div data-tauri-drag-region className="relative z-10 space-y-4">
        {/* Top Row: Title */}
        <div className="flex items-center justify-between">
          <h1 data-tauri-drag-region className="text-2xl font-bold tracking-tight">
            设置
          </h1>
        </div>

        {/* Tabs Scroll Area */}
        <div className="flex items-center gap-2 overflow-x-auto no-scrollbar pb-2 -mx-8 px-8 mask-linear-fade">
          {categories.map((category) => {
            const isActive = activeCategory === category.id;

            return (
              <motion.button
                key={category.id}
                onClick={() => onCategoryClick(category.id)}
                className={cn(
                  "relative group flex items-center justify-center px-4 py-2 rounded-xl text-sm font-medium whitespace-nowrap transition-all duration-300 outline-none select-none",
                  isActive
                    ? "text-primary-foreground"
                    : "text-muted-foreground hover:text-foreground hover:bg-muted/50"
                )}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.96 }}
              >
                {isActive && (
                  <motion.div
                    layoutId="activeSettingTab"
                    className="absolute inset-0 bg-primary rounded-xl shadow-lg shadow-primary/25"
                    transition={{ type: "spring", bounce: 0.2, duration: 0.6 }}
                  />
                )}
                <span className="relative z-10">{category.name}</span>
              </motion.button>
            );
          })}
        </div>
      </div>
    </header>
  );
};

export default SettingHeader;
