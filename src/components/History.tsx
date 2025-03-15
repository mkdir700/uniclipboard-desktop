import React from 'react';
import HistoryHeader from './HistoryHeader';
import HistoryContent from './HistoryContent';

const History: React.FC = () => {
  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* 顶部搜索栏 */}
      <HistoryHeader />
      
      {/* 主内容区域 - 滚动容器 */}
      <div className="flex-1 overflow-y-auto hide-scrollbar p-6">
        <HistoryContent />
      </div>
    </div>
  );
};

export default History;
