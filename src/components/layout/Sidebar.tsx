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
import { useTranslation } from "react-i18next";

const Sidebar: React.FC = () => {
  const { t } = useTranslation();
  const location = useLocation();
  const path = location.pathname;

  const navItems = [
    { to: "/", icon: Home, label: t("nav.dashboard") },
    { to: "/devices", icon: Monitor, label: t("nav.devices") },
  ];

  const bottomItems = [
    { to: "/settings", icon: Settings, label: t("nav.settings") },
  ];

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
            <Link
              data-tauri-drag-region="false"
              to={to}
              className="relative group"
            >
              {isActive && (
                <motion.div
                  layoutId={layoutId}
                  className="absolute inset-0 bg-primary/10 dark:bg-primary/20 rounded-lg"
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
                  "relative flex items-center justify-center w-12 h-12 rounded-lg transition-colors duration-200 z-10",
                  isActive
                    ? "text-primary"
                    : "text-muted-foreground group-hover:text-primary group-hover:bg-muted"
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
      className="w-16 h-screen sticky top-0 z-100 flex flex-col items-center pt-10 pb-6 bg-muted/40 border-r border-border/40 backdrop-blur-xl shrink-0"
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

      <div data-tauri-drag-region className="flex-1 w-full" />

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
