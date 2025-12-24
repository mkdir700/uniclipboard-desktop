import React from "react";
import { Link, useLocation } from "react-router-dom";
import { Home, Monitor, Settings, Info } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

const Sidebar: React.FC = () => {
  const location = useLocation();
  const path = location.pathname;

  const navItems = [
    { to: "/", icon: Home, label: "仪表板" },
    { to: "/devices", icon: Monitor, label: "设备管理" },
  ];

  const bottomItems = [
    { to: "/about", icon: Info, label: "关于" },
    { to: "/settings", icon: Settings, label: "设置" },
  ];

  const NavButton: React.FC<{
    to: string;
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    isActive: boolean;
  }> = ({ to, icon: Icon, label, isActive }) => {
    return (
      <TooltipProvider delayDuration={0}>
        <Tooltip>
          <TooltipTrigger asChild>
            <Link to={to}>
              <Button
                variant="ghost"
                size="icon"
                className={cn(
                  "h-12 w-12 rounded-2xl transition-all duration-200",
                  isActive
                    ? "bg-primary/10 text-primary dark:bg-primary/20"
                    : "text-muted-foreground hover:text-primary hover:bg-accent"
                )}
              >
                <Icon className="h-6 w-6" />
              </Button>
            </Link>
          </TooltipTrigger>
          <TooltipContent side="right">
            <p>{label}</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  };

  return (
    <aside className="w-20 bg-card border-r border-border flex flex-col items-center py-6 gap-8 shrink-0 z-20">
      {/* Logo / Home */}
      <NavButton to="/" icon={Home} label="首页" isActive={path === "/"} />

      {/* Main Navigation */}
      <div className="flex flex-col gap-2">
        {navItems.map((item) => (
          <NavButton
            key={item.to}
            to={item.to}
            icon={item.icon}
            label={item.label}
            isActive={path === item.to}
          />
        ))}
      </div>

      <div className="flex-1"></div>

      {/* Bottom Navigation */}
      <div className="flex flex-col gap-2">
        {bottomItems.map((item) => (
          <NavButton
            key={item.to}
            to={item.to}
            icon={item.icon}
            label={item.label}
            isActive={path === item.to}
          />
        ))}
      </div>
    </aside>
  );
};

export default Sidebar;
