import React, { useState, useEffect, useRef, useCallback } from "react";
import Header from "@/components/layout/Header";
import ClipboardContent from "@/components/clipboard/ClipboardContent";
import ActionBar from "@/components/layout/ActionBar";
import { MainLayout } from "@/layouts";
import { Filter, OrderBy } from "@/api/clipboardItems";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { fetchStats } from "@/store/slices/statsSlice";
import { fetchClipboardItems } from "@/store/slices/clipboardSlice";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// 防抖延迟时间（毫秒）
const DEBOUNCE_DELAY = 500;

// 全局监听器状态管理
interface ListenerState {
  isActive: boolean;
  unlisten?: () => void;
  lastEventTimestamp?: number;
}

const globalListenerState: ListenerState = {
  isActive: false,
};

const DashboardPage: React.FC = () => {
  const [currentFilter, setCurrentFilter] = useState<Filter>(Filter.All);
  const statsState = useAppSelector((state) => state.stats);
  const dispatch = useAppDispatch();

  // 使用ref保存最新的filter值
  const currentFilterRef = useRef<Filter>(currentFilter);

  // 防抖引用
  const debouncedLoadRef = useRef<number | null>(null);

  const handleFilterChange = (filterId: Filter) => {
    setCurrentFilter(filterId);
  };

  // 加载剪贴板记录和统计数据
  const loadData = useCallback(
    async (specificFilter?: Filter) => {
      const filterToUse = specificFilter || currentFilterRef.current;
      console.log("开始加载剪贴板记录和统计数据...", filterToUse);

      dispatch(
        fetchClipboardItems({
          orderBy: OrderBy.ActiveTimeDesc,
          filter: filterToUse,
        })
      );

      dispatch(fetchStats());
    },
    [dispatch]
  );

  // 处理同步按钮点击事件
  const handleSync = useCallback(() => {
    // 同步完成后立即加载最新数据
    loadData(currentFilterRef.current);
  }, [loadData]);

  // 防抖处理数据加载
  const debouncedLoadData = useCallback(
    (specificFilter?: Filter) => {
      if (debouncedLoadRef.current) {
        clearTimeout(debouncedLoadRef.current);
      }

      debouncedLoadRef.current = setTimeout(() => {
        loadData(specificFilter);
        debouncedLoadRef.current = null;
      }, DEBOUNCE_DELAY);
    },
    [loadData]
  );

  // 更新ref以跟踪最新的filter
  useEffect(() => {
    console.log("filter变化，更新ref值:", currentFilter);
    currentFilterRef.current = currentFilter;
    loadData(currentFilter); // 直接加载，不用防抖
  }, [currentFilter, loadData]);

  // 设置剪贴板内容监听器
  useEffect(() => {
    // 设置监听器的函数
    const setupListener = async () => {
      // 只有在还没有活跃的监听器时才设置
      if (!globalListenerState.isActive) {
        console.log("设置全局监听器...");
        globalListenerState.isActive = true;

        try {
          console.log("启动后端剪贴板新内容监听...");
          await invoke("listen_clipboard_new_content");
          console.log("后端剪贴板新内容监听已启动");

          console.log("开始监听剪贴板新内容事件...");
          // 清除之前可能存在的监听器
          if (globalListenerState.unlisten) {
            console.log("清除之前的监听器");
            globalListenerState.unlisten();
            globalListenerState.unlisten = undefined;
          }

          // 使用listen函数监听全局事件
          const unlisten = await listen<{
            record_id: string;
            timestamp: number;
          }>("clipboard-new-content", (event) => {
            console.log("收到新剪贴板内容事件:", event);

            // 检查事件时间戳，避免短时间内重复处理同一事件
            const currentTime = Date.now();
            if (
              globalListenerState.lastEventTimestamp &&
              currentTime - globalListenerState.lastEventTimestamp <
                DEBOUNCE_DELAY
            ) {
              console.log("忽略短时间内的重复事件");
              return;
            }

            // 更新最后事件时间戳
            globalListenerState.lastEventTimestamp = currentTime;

            // 使用防抖函数加载数据
            debouncedLoadData(currentFilterRef.current);
          });

          // 保存解除监听的函数到全局状态
          globalListenerState.unlisten = unlisten;
        } catch (err) {
          console.error("设置监听器失败:", err);
          globalListenerState.isActive = false;
        }
      } else {
        console.log("监听器已经处于活跃状态，跳过设置");
      }
    };

    // 如果还没有设置监听器，则设置
    if (!globalListenerState.isActive) {
      setupListener();
    } else {
      console.log("全局监听器已存在，无需再次设置");
    }

    // 组件卸载时的清理函数
    return () => {
      // 清除防抖定时器
      if (debouncedLoadRef.current) {
        clearTimeout(debouncedLoadRef.current);
      }
      // 不清理全局监听器，让它持续存在
      console.log("组件卸载，但保持全局监听器活跃");
    };
  }, [debouncedLoadData]);

  return (
    <MainLayout>
      {/* 顶部搜索栏 */}
      <Header onFilterChange={handleFilterChange} />

      {/* 剪贴板内容区域 */}
      <ClipboardContent filter={currentFilter} />

      {/* 底部快捷操作栏 */}
      <ActionBar stats={statsState.stats} onSync={handleSync} />
    </MainLayout>
  );
};

export default DashboardPage;
