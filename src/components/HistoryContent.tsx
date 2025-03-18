import React, { useState, useEffect } from 'react';
import ClipboardItem from './ClipboardItem';
import DateGroup from './DateGroup';
import { getClipboardItems, ClipboardItemResponse, getDisplayType, deleteClipboardItem } from '../api/clipboardItems';

// 每次加载的条目数量
const PAGE_SIZE = 20;

// 对历史记录按日期进行分组
interface GroupedItems {
  [date: string]: {
    items: ClipboardItemResponse[];
    dateStr: string;
    description: string;
  };
}

// 日期信息结构
interface DateInfo {
  dateStr: string;
  description: string;
}

// 将时间戳转为日期字符串
const formatDate = (timestamp: number): DateInfo => {
  const date = new Date(timestamp * 1000);
  const today = new Date();
  const yesterday = new Date();
  yesterday.setDate(yesterday.getDate() - 1);
  
  // 格式化为 "MM月DD日" 格式
  const month = date.getMonth() + 1;
  const day = date.getDate();
  const formattedDate = `${month}月${day}日`;
  
  // 判断是否为今天或昨天
  if (
    date.getDate() === today.getDate() &&
    date.getMonth() === today.getMonth() &&
    date.getFullYear() === today.getFullYear()
  ) {
    return {
      dateStr: formattedDate,
      description: '今天'
    };
  } else if (
    date.getDate() === yesterday.getDate() &&
    date.getMonth() === yesterday.getMonth() &&
    date.getFullYear() === yesterday.getFullYear()
  ) {
    return {
      dateStr: formattedDate,
      description: '昨天'
    };
  }
  
  // 如果不是今天或昨天，返回星期几
  const weekDays = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六'];
  return {
    dateStr: formattedDate,
    description: weekDays[date.getDay()]
  };
};

// 获取月份标题（YYYY年MM月）
const getMonthHeader = (timestamp: number): string => {
  const date = new Date(timestamp * 1000);
  return `${date.getFullYear()}年${date.getMonth() + 1}月`;
};

const HistoryContent: React.FC = () => {
  // 状态管理
  const [items, setItems] = useState<ClipboardItemResponse[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [offset, setOffset] = useState<number>(0);
  const [hasMore, setHasMore] = useState<boolean>(true);
  const [groupedItems, setGroupedItems] = useState<GroupedItems>({});

  // 加载剪贴板历史记录
  const loadItems = async (resetItems: boolean = false) => {
    if (loading && !resetItems) return;
    
    setLoading(true);
    setError(null);
    
    try {
      const newOffset = resetItems ? 0 : offset;
      const result = await getClipboardItems(PAGE_SIZE, newOffset);
      
      // 更新状态
      if (resetItems) {
        setItems(result);
      } else {
        setItems(prev => [...prev, ...result]);
      }
      
      setOffset(newOffset + result.length);
      setHasMore(result.length === PAGE_SIZE);
    } catch (err) {
      setError('获取剪贴板历史记录失败');
      console.error('获取剪贴板历史记录失败:', err);
    } finally {
      setLoading(false);
    }
  };

  // 处理删除
  const handleDelete = async (id: string) => {
    try {
      await deleteClipboardItem(id);
      // 从列表中移除已删除的项目
      setItems(prev => prev.filter(item => item.id !== id));
    } catch (err) {
      console.error('删除剪贴板项目失败:', err);
    }
  };

  // 处理复制
  const handleCopy = async (item: ClipboardItemResponse) => {
    try {
      // 使用内置API复制内容
      await navigator.clipboard.writeText(item.display_content);
      return true;
    } catch (err) {
      console.error('复制内容失败:', err);
      return false;
    }
  };

  // 首次加载
  useEffect(() => {
    loadItems(true);
  }, []);

  // 处理数据分组
  useEffect(() => {
    if (items.length === 0) return;
    
    const grouped: GroupedItems = {};
    
    // 按日期分组
    items.forEach(item => {
      const dateInfo = formatDate(item.created_at);
      const dateKey = new Date(item.created_at * 1000).toISOString().substring(0, 10); // YYYY-MM-DD
      
      if (!grouped[dateKey]) {
        grouped[dateKey] = {
          items: [],
          dateStr: dateInfo.dateStr,
          description: dateInfo.description
        };
      }
      
      grouped[dateKey].items.push(item);
    });
    
    // 更新状态
    setGroupedItems(grouped);
  }, [items]);

  // 渲染内容
  return (
    <div className="max-w-4xl mx-auto">
      {/* 错误提示 */}
      {error && (
        <div className="text-red-500 mb-4 bg-red-50 p-3 rounded">{error}</div>
      )}
      
      {/* 无内容提示 */}
      {!loading && items.length === 0 && (
        <div className="text-center py-10 text-gray-500">
          <p>暂无剪贴板历史记录</p>
        </div>
      )}
      
      {/* 渲染分组内容 */}
      {Object.entries(groupedItems).map(([dateKey, group]) => (
        <DateGroup 
          key={dateKey} 
          date={group.dateStr} 
          description={group.description}
        >
          {/* 渲染组内项目 */}
          {group.items.map(item => (
            <ClipboardItem
              key={item.id}
              type={getDisplayType(item.content_type)}
              content={item.content_type === 'image' ? '' : item.display_content}
              imageUrl={item.content_type === 'image' ? item.display_content : undefined}
              time={new Date(item.created_at * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
              device={item.device_id}
              isDownloaded={item.is_downloaded}
              onDelete={() => handleDelete(item.id)}
              onCopy={() => handleCopy(item)}
            />
          ))}
        </DateGroup>
      ))}
      
      {/* 加载更多 */}
      {hasMore && (
        <div className="mt-4">
          <button 
            className={`w-full text-center py-3 text-gray-500 hover:bg-gray-100 rounded-md ${loading ? 'opacity-50 cursor-not-allowed' : ''}`}
            onClick={() => loadItems()}
            disabled={loading}
          >
            {loading ? '加载中...' : '加载更多'}
          </button>
        </div>
      )}
    </div>
  );
};

export default HistoryContent;
