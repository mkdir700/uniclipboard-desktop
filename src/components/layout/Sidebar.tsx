
import React from "react";
import { Link, useLocation } from "react-router-dom";
import { Home, Monitor, Settings } from "lucide-react";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { motion } from "framer-motion";

const Sidebar: React.FC = () => {
  const location = useLocation();
  const path = location.pathname;

  const navItems = [
    { to: "/", icon: Home, label: "仪表板" },
    { to: "/devices", icon: Monitor, label: "设备管理" },
  ];

  const bottomItems = [{ to: "/settings", icon: Settings, label: "设置" }];

  const NavButton: React.FC<{
    to: string;
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    isActive: boolean;
    layoutId: string;
  }> = ({ to, icon: Icon, label, isActive, layoutId }) => {
    return (
      <TooltipProvider delayDuration={0}>
        <Tooltip>
          <TooltipTrigger asChild>
            <Link to={to} className="relative group">
              {isActive && (
                <motion.div
                  layoutId={layoutId}
                  className="absolute inset-0 bg-primary/10 dark:bg-primary/20 rounded-xl"
                  initial={false}
                  transition={{
                    type: "spring",
                    stiffness: 500,
                    damping: 30,
                  }}
                />
              )}
              <div
                className={cn(
                  "relative flex items-center justify-center w-12 h-12 rounded-xl transition-colors duration-200 z-10",
                  isActive
                    ? "text-primary"
                    : "text-muted-foreground group-hover:text-primary group-hover:bg-accent/50"
                )}
              >
                <Icon className="w-5 h-5" />
              </div>
            </Link>
          </TooltipTrigger>
          <TooltipContent side="right" className="ml-2 font-medium">
            <p>{label}</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  };

  return (
    <aside 
      data-tauri-drag-region
      className="w-[72px] h-screen sticky top-0 z-[100] flex flex-col items-center pt-10 pb-6 bg-muted/40 border-r border-border/40 backdrop-blur-xl shrink-0"
    >
      {/* Main Navigation */}
      <div className="flex flex-col gap-3 w-full items-center pt-2">
        {navItems.map((item) => (
          <NavButton
            key={item.to}
            to={item.to}
            icon={item.icon}
            label={item.label}
            isActive={path === item.to}
            layoutId="sidebar-nav-top"
          />
        ))}
      </div>

      <div className="flex-1" />

      {/* Bottom Navigation */}
      <div className="flex flex-col gap-3 w-full items-center pb-2">
        {bottomItems.map((item) => (
          <NavButton
            key={item.to}
            to={item.to}
            icon={item.icon}
            label={item.label}
            isActive={path === item.to}
            layoutId="sidebar-nav-bottom"
          />
        ))}
      </div>
    </aside>
  );
};

export default Sidebar;
