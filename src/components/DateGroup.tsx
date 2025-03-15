import React from "react";

interface DateGroupProps {
  date: string;
  description?: string;
  children: React.ReactNode;
}

const DateGroup: React.FC<DateGroupProps> = ({
  date,
  description,
  children,
}) => {
  return (
    <div className="mb-6">
      {/* 日期标题 */}
      <div className="mb-3 flex items-center">
        <span className="text-xs font-medium bg-gray-800/60 px-2 py-1 rounded text-gray-400">
          {date}
          {description && ` · ${description}`}
        </span>
        <div className="ml-2 flex-grow border-t border-gray-800/30"></div>
      </div>

      <div className="space-y-2">{children}</div>
    </div>
  );
};

export default DateGroup;
