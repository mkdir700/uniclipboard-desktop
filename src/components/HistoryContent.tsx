import React from 'react';
import ClipboardItem from './ClipboardItem';
import DateGroup from './DateGroup';
import MonthHeader from './MonthHeader';
import LoadingMore from './LoadingMore';

const HistoryContent: React.FC = () => {
  return (
    <div className="max-w-4xl mx-auto">
      {/* 今日分组 */}
      <DateGroup date="10月28日" description="今天">
        {/* 文本项目 - 使用通用 ClipboardItem 组件 */}
        <ClipboardItem 
          type="text"
          title="文本内容"
          content="这是一段示例文本内容，用于演示历史记录中的文本条目。这可能是用户复制的一段文字、一个笔记或者其他文本内容。"
          time="14:32"
          device="MacBook Pro"
        />
        
        {/* 图片项目 */}
        <ClipboardItem 
          type="image"
          title="图片内容"
          content="图片内容"
          time="12:05"
          device="iPhone 13"
          imageUrl="https://images.unsplash.com/photo-1682687982501-1e58ab814714"
        />
      </DateGroup>
      
      {/* 另一个日期分组 */}
      <DateGroup date="10月27日" description="星期五">
        {/* 链接项目 */}
        <ClipboardItem 
          type="link"
          title="React 官方文档"
          content="https://react.dev/learn"
          time="16:45"
          device="Chrome"
        />
        
        {/* 代码项目 */}
        <ClipboardItem 
          type="code"
          title="同步函数代码"
          content={`async function syncToDevices(text) {
  try {
    const response = await fetch('/api/sync', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content: text, type: 'text' })
    });
    return await response.json();
  } catch (error) {
    console.error('同步失败:', error);
  }
}`}
          time="11:20"
          device="VS Code"
        />
        
        {/* 文件项目 */}
        <ClipboardItem 
          type="file"
          title="项目报告.pdf"
          content="2.4 MB"
          time="09:15"
          device="Chrome"
        />
      </DateGroup>
      
      {/* 月份标题 */}
      <MonthHeader month="2023年9月" />
      
      {/* 9月份分组 */}
      <DateGroup date="9月30日" description="星期六">
        {/* 使用 ClipboardItem 组件，但传入自定义类型 */}
        <ClipboardItem 
          type="link"
          title="地址信息"
          content="上海市浦东新区张江高科技园区博云路2号"
          time="18:30"
          device="高德地图"
        />
      </DateGroup>
      
      {/* 懒加载提示 */}
      <LoadingMore />
    </div>
  );
};

export default HistoryContent;
