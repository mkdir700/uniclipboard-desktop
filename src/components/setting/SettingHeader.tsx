import React from "react";

export interface CategoryItem {
  id: string;
  name: string;
}

interface SettingHeaderProps {
  onCategoryClick: (category: string) => void;
  activeCategory: string;
  categories: CategoryItem[];
}

const SettingHeader: React.FC<SettingHeaderProps> = ({
  onCategoryClick,
  activeCategory,
  categories,
}) => {
  return (
    <>
      <header className="bg-gray-900 border-b border-gray-800/50">
        <div className="px-4 py-4 flex items-center justify-between">
          <h1 className="text-xl font-semibold text-white">设置</h1>
        </div>

        {/* 设置类别标签 */}
        <div className="px-4 pb-2 flex space-x-4 text-sm overflow-x-auto hide-scrollbar">
          {categories.map((category) => (
            <button
              key={category.id}
              className={`${
                activeCategory === category.id
                  ? "text-white border-violet-400"
                  : "text-gray-400 hover:text-white border-transparent"
              } pb-2 border-b-2 whitespace-nowrap transition-colors duration-200`}
              onClick={() => onCategoryClick(category.id)}
            >
              {category.name}
            </button>
          ))}
        </div>
      </header>
    </>
  );
};

export default SettingHeader;
