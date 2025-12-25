import React, { ReactNode } from "react";

interface SettingContentLayoutProps {
  title: string;
  children: ReactNode;
}

const SettingContentLayout: React.FC<SettingContentLayoutProps> = ({ 
  title, 
  children 
}) => {
  return (
    <div className="mb-8">
      <div className="flex items-center gap-4 mb-4">
        <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">{title}</h3>
        <div className="h-px flex-1 bg-border/50"></div>
      </div>
      
      <div className="bg-card border border-border/50 rounded-xl p-6 shadow-sm">
        <div className="space-y-6">
          {children}
        </div>
      </div>
    </div>
  );
};

export default SettingContentLayout;
