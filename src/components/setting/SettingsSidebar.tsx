import React from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import {
  Settings,
  Palette,
  RefreshCw,
  Shield,
  Wifi,
  HardDrive,
  Info,
  ArrowLeft,
} from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuItem,
  SidebarHeader,
} from "@/components/ui/sidebar";

interface SettingsSidebarProps {
  activeCategory: string;
  onCategoryChange: (category: string) => void;
}

const SettingsSidebar: React.FC<SettingsSidebarProps> = ({
  activeCategory,
  onCategoryChange,
}) => {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const handleBack = () => {
    navigate(-1);
  };

  const settingsNavItems = [
    {
      id: "general",
      label: t("settings.categories.general"),
      icon: Settings,
    },
    {
      id: "appearance",
      label: t("settings.categories.appearance"),
      icon: Palette,
    },
    {
      id: "sync",
      label: t("settings.categories.sync"),
      icon: RefreshCw,
    },
    {
      id: "security",
      label: t("settings.categories.security"),
      icon: Shield,
    },
    {
      id: "network",
      label: t("settings.categories.network"),
      icon: Wifi,
    },
    {
      id: "storage",
      label: t("settings.categories.storage"),
      icon: HardDrive,
    },
    {
      id: "about",
      label: t("settings.categories.about"),
      icon: Info,
    },
  ];

  return (
    <Sidebar collapsible="none" className="border-r border-border/50 bg-muted/30">
      <SidebarHeader className="border-b border-border/50">
        <button
          onClick={handleBack}
          className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm font-medium outline-none ring-sidebar-ring transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-2 active:bg-muted active:text-foreground"
        >
          <ArrowLeft className="h-4 w-4 shrink-0" />
          <span>{t("settings.title")}</span>
        </button>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {settingsNavItems.map((item) => {
                const Icon = item.icon;
                const isActive = activeCategory === item.id;

                return (
                  <SidebarMenuItem key={item.id}>
                    <button
                      onClick={() => onCategoryChange(item.id)}
                      className={`flex w-full items-center gap-2 overflow-hidden rounded-md p-2 text-left text-sm outline-none ring-sidebar-ring transition-[width,height,padding] focus-visible:ring-2 disabled:pointer-events-none disabled:opacity-50 [&>span:last-child]:truncate [&>svg]:size-4 [&>svg]:shrink-0 ${
                        isActive
                          ? 'bg-muted/50 font-medium text-foreground'
                          : 'text-muted-foreground hover:bg-muted hover:text-foreground'
                      }`}
                    >
                      <Icon className="h-4 w-4" />
                      <span>{item.label}</span>
                    </button>
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
};

export default SettingsSidebar;
