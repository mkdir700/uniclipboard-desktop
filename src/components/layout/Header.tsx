import React, { useState } from "react";
import { Search, ClipboardCopy, Star, FileText, Image, Link as LinkIcon, Folder, Code } from "lucide-react";
import { Filter } from "@/api/clipboardItems";
import { Input } from "@/components/ui/input";
import { motion } from "framer-motion";
import { cn } from "@/lib/utils";
import { useTranslation } from "react-i18next";

interface HeaderProps {
  onFilterChange?: (filterId: Filter) => void;
}

const Header: React.FC<HeaderProps> = ({ onFilterChange }) => {
  const { t } = useTranslation();

  const filterTypes = [
    { id: Filter.All, label: "header.filters.all", icon: ClipboardCopy },
    { id: Filter.Favorited, label: "header.filters.favorited", icon: Star },
    { id: Filter.Text, label: "header.filters.text", icon: FileText },
    { id: Filter.Image, label: "header.filters.image", icon: Image },
    { id: Filter.Link, label: "header.filters.link", icon: LinkIcon },
    { id: Filter.File, label: "header.filters.file", icon: Folder },
    { id: Filter.Code, label: "header.filters.code", icon: Code },
  ];

  const [activeFilter, setActiveFilter] = useState<Filter>(Filter.All);
  const [isSearchFocused, setIsSearchFocused] = useState(false);

  const handleFilterClick = (filterId: Filter) => {
    setActiveFilter(filterId);
    onFilterChange?.(filterId);
  };

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
        {/* Top Row: Search & Status */}
        <div className="flex items-center justify-between gap-4">
          <motion.div 
            className={cn(
              "relative flex-1 group transition-all duration-300",
              isSearchFocused ? "scale-[1.01]" : ""
            )}
            initial={false}
          >
            <div className={cn(
              "absolute inset-0 bg-gradient-to-r from-primary/20 to-secondary/20 rounded-lg blur-md transition-opacity duration-500",
              isSearchFocused ? "opacity-100" : "opacity-0"
            )} />
            <div className={cn(
              "relative flex items-center px-4 py-3 bg-card/50 backdrop-blur-md rounded-lg border transition-all duration-300",
              isSearchFocused
                ? "border-transparent shadow-lg shadow-primary/5"
                : "border-border/50 shadow-sm hover:border-border/80 hover:bg-card/80"
            )}>
              <Search className={cn(
                "h-5 w-5 mr-3 transition-colors duration-300",
                isSearchFocused ? "text-primary" : "text-muted-foreground"
              )} />
              <Input
                data-tauri-drag-region="false"
                type="text"
                placeholder={t("header.searchPlaceholder")}
                className="bg-transparent border-none p-0 h-auto focus-visible:ring-0 focus-visible:ring-offset-0 placeholder:text-muted-foreground/50"
                onFocus={() => setIsSearchFocused(true)}
                onBlur={() => setIsSearchFocused(false)}
              />
            </div>
          </motion.div>


        </div>

        {/* Filter Scroll Area */}
        <div className="flex items-center gap-2 overflow-x-auto no-scrollbar pb-2 -mx-8 px-8 mask-linear-fade">
          {filterTypes.map((filter) => {
            const Icon = filter.icon;
            const isActive = activeFilter === filter.id;

            return (
              <motion.button
                data-tauri-drag-region="false"
                key={filter.id}
                onClick={() => handleFilterClick(filter.id)}
                className={cn(
                  "relative group flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium whitespace-nowrap transition-all duration-300 outline-none select-none",
                  isActive ? "text-primary-foreground" : "text-muted-foreground hover:text-foreground hover:bg-muted/50"
                )}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.96 }}
              >
                {isActive && (
                  <motion.div
                    layoutId="activeFilter"
                    className="absolute inset-0 bg-primary rounded-lg shadow-lg shadow-primary/25"
                    transition={{ type: "spring", bounce: 0.2, duration: 0.6 }}
                  />
                )}
                <span className="relative z-10 flex items-center gap-2">
                  <Icon className={cn("h-4 w-4", isActive ? "text-primary-foreground" : "")} />
                  {t(filter.label)}
                </span>
              </motion.button>
            );
          })}
        </div>
      </div>
    </header>
  );
};

export default Header;
