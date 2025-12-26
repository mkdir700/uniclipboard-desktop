import React from "react";
import { useTranslation } from "react-i18next";
import {
  Settings,
  Palette,
  RefreshCw,
  Shield,
  Wifi,
  HardDrive,
  Info,
} from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
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
    <Sidebar>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {settingsNavItems.map((item) => {
                const Icon = item.icon;
                const isActive = activeCategory === item.id;

                return (
                  <SidebarMenuItem key={item.id}>
                    <SidebarMenuButton
                      asChild
                      isActive={isActive}
                      onClick={() => onCategoryChange(item.id)}
                    >
                      <button>
                        <Icon className="h-4 w-4" />
                        <span>{item.label}</span>
                      </button>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarRail />
    </Sidebar>
  );
};

export default SettingsSidebar;
