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
    <div className="mb-6">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-sm font-medium text-gray-400">{title}</h3>
        <div className="flex-grow ml-3 border-t border-gray-800/50"></div>
      </div>
      
      <div className="bg-gray-800 rounded-lg p-4 mb-4">
        {children}
      </div>
    </div>
  );
};

export default SettingContentLayout;
