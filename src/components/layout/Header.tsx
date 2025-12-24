import React, { useState } from "react";
import { Search, ClipboardCopy, Star, FileText, Image, Link as LinkIcon, Folder, Code } from "lucide-react";
import { Filter } from "@/api/clipboardItems";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

interface HeaderProps {
  onFilterChange?: (filterId: Filter) => void;
}

const Header: React.FC<HeaderProps> = ({ onFilterChange }) => {
  // 定义筛选类型
  const filterTypes = [
    { id: Filter.All, label: "全部", icon: ClipboardCopy },
    { id: Filter.Favorited, label: "收藏", icon: Star },
    { id: Filter.Text, label: "文本", icon: FileText },
    { id: Filter.Image, label: "图片", icon: Image },
    { id: Filter.Link, label: "链接", icon: LinkIcon },
    { id: Filter.File, label: "文件", icon: Folder },
    { id: Filter.Code, label: "代码", icon: Code },
  ];

  // 当前选中的筛选类型
  const [activeFilter, setActiveFilter] = useState<Filter>(Filter.All);

  // 处理筛选器点击
  const handleFilterClick = (filterId: Filter) => {
    setActiveFilter(filterId);
    // 调用父组件传入的回调函数
    onFilterChange?.(filterId);
  };

  return (
    <header className="px-8 pt-6 pb-2 shrink-0">
      {/* 搜索栏 */}
      <div className="bg-card rounded-full shadow-sm border border-border flex items-center px-4 py-3">
        <Search className="h-5 w-5 text-muted-foreground mr-3" />
        <Input
          type="text"
          placeholder="搜索剪贴板内容..."
          className="bg-transparent border-none focus-visible:ring-0 focus-visible:ring-offset-0 text-sm w-full"
        />
        {/* 同步状态指示器 */}
        <Badge
          variant="secondary"
          className="bg-success/10 text-success hover:bg-success/20 px-3 py-1 rounded-full text-xs font-medium ml-2 shrink-0"
        >
          <span>已同步</span>
          <div className="w-2 h-2 rounded-full bg-success ml-2"></div>
        </Badge>
      </div>

      {/* 内容类型筛选器 */}
      <div className="mt-6 pb-2 flex gap-3 overflow-x-auto no-scrollbar shrink-0 items-center">
        {filterTypes.map((filter) => {
          const Icon = filter.icon;
          const isActive = activeFilter === filter.id;
          return (
            <Button
              key={filter.id}
              variant={isActive ? "secondary" : "ghost"}
              size="sm"
              onClick={() => handleFilterClick(filter.id)}
              className={cn(
                "flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-medium whitespace-nowrap transition-all shadow-sm",
                isActive
                  ? filter.id === Filter.Favorited
                    ? "bg-amber-100 text-amber-700 hover:bg-amber-200 dark:bg-amber-900/30 dark:text-amber-400 border border-amber-200 dark:border-amber-800"
                    : "bg-primary/10 text-primary hover:bg-primary/20 dark:bg-primary/20 border border-primary/20"
                  : "bg-card text-muted-foreground hover:bg-accent hover:text-foreground border border-border"
              )}
            >
              <Icon className="h-4 w-4" />
              {filter.label}
            </Button>
          );
        })}
      </div>
    </header>
  );
};

export default Header;
