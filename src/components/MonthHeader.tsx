import React from "react";

interface MonthHeaderProps {
  month: string;
}

const MonthHeader: React.FC<MonthHeaderProps> = ({ month }) => {
  return (
    <div className="sticky top-0 z-10 flex items-center pt-6 pb-3">
      <h3 className="bg-gray-800 text-sm font-semibold text-white px-3 py-1 rounded-full">
        {month}
      </h3>
      <div className="ml-3 flex-grow border-t border-gray-800/30"></div>
    </div>
  );
};

export default MonthHeader;
