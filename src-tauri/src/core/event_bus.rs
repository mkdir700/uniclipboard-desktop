use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

/// 代表一个事件监听器的ID，用于后续取消监听
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId(pub(crate) usize);

/// 事件总线系统的核心结构
#[derive(Default)]
pub struct EventBus {
    // 使用TypeId索引不同类型的事件，每种事件类型都有一组监听器
    listeners: RwLock<HashMap<TypeId, Vec<(ListenerId, Arc<dyn Fn(&dyn Any) + Send + Sync>)>>>,
    next_listener_id: Mutex<usize>,
}

impl EventBus {
    /// 创建一个新的事件总线实例
    pub fn new() -> Self {
        Self {
            listeners: RwLock::new(HashMap::new()),
            next_listener_id: Mutex::new(0),
        }
    }

    /// 订阅特定类型的事件
    /// 
    /// 泛型参数:
    /// - `E`: 事件类型，必须是'static和Clone
    /// - `F`: 回调函数，接受事件实例并处理
    /// 
    /// 返回:
    /// - `ListenerId`: 唯一的监听器ID，可用于取消订阅
    pub fn subscribe<E, F>(&self, callback: F) -> ListenerId
    where
        E: 'static + Clone,
        F: Fn(&E) + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<E>();
        
        // 生成唯一的监听器ID
        let id = {
            let mut id_guard = self.next_listener_id.lock().unwrap();
            let id = ListenerId(*id_guard);
            *id_guard += 1;
            id
        };
        
        // 将类型特定的回调转换为接受任意类型的回调
        let any_callback: Arc<dyn Fn(&dyn Any) + Send + Sync> = Arc::new(move |any| {
            if let Some(event) = any.downcast_ref::<E>() {
                callback(event);
            }
        });
        
        // 存储监听器
        let mut listeners = self.listeners.write().unwrap();
        listeners
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push((id, any_callback));
            
        id
    }

    /// 取消订阅特定的事件监听器
    ///
    /// 参数:
    /// - `id`: 要取消的监听器ID
    ///
    /// 返回:
    /// - `bool`: 如果找到并移除了监听器则返回true，否则返回false
    pub fn unsubscribe(&self, id: ListenerId) -> bool {
        let mut listeners = self.listeners.write().unwrap();
        let mut found = false;
        
        // 遍历所有事件类型，查找并移除指定ID的监听器
        for (_type_id, type_listeners) in listeners.iter_mut() {
            let before_len = type_listeners.len();
            type_listeners.retain(|(listener_id, _)| *listener_id != id);
            if type_listeners.len() < before_len {
                found = true;
            }
        }
        
        found
    }

    /// 发布事件到所有相关的监听器
    /// 
    /// 泛型参数:
    /// - `E`: 事件类型，必须是'static和Clone
    /// 
    /// 参数:
    /// - `event`: 要发布的事件实例
    pub fn publish<E>(&self, event: E)
    where
        E: 'static + Clone,
    {
        let type_id = TypeId::of::<E>();
        
        // 获取关联此事件类型的所有监听器
        let listeners = self.listeners.read().unwrap();
        if let Some(type_listeners) = listeners.get(&type_id) {
            // 克隆事件并通知所有监听器
            for (_, callback) in type_listeners {
                callback(&event);
            }
        }
    }
}

/// 全局事件总线实例
lazy_static::lazy_static! {
    pub static ref EVENT_BUS: EventBus = EventBus::new();
}

/// 剪贴板新内容事件
#[derive(Clone)]
pub struct ClipboardNewContentEvent {
    /// 新增内容的记录ID
    pub record_id: String,
    /// 事件发生的时间戳（毫秒）
    pub timestamp: u64,
}

/// 便捷函数：发布剪贴板新内容事件
pub fn publish_clipboard_new_content(record_id: String) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
        
    let event = ClipboardNewContentEvent {
        record_id,
        timestamp,
    };
    
    EVENT_BUS.publish(event);
}

/// 便捷函数：订阅剪贴板新内容事件
pub fn subscribe_clipboard_new_content<F>(callback: F) -> ListenerId
where
    F: Fn(&ClipboardNewContentEvent) + Send + Sync + 'static,
{
    EVENT_BUS.subscribe(callback)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    
    #[test]
    fn test_subscribe_and_publish() {
        let event_bus = EventBus::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        
        #[derive(Clone)]
        struct TestEvent(i32);
        
        let _id = event_bus.subscribe::<TestEvent, _>(move |event| {
            assert_eq!(event.0, 42);
            called_clone.store(true, Ordering::SeqCst);
        });
        
        event_bus.publish(TestEvent(42));
        assert!(called.load(Ordering::SeqCst));
    }
    
    #[test]
    fn test_unsubscribe() {
        let event_bus = EventBus::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        
        #[derive(Clone)]
        struct TestEvent(i32);
        
        let id = event_bus.subscribe::<TestEvent, _>(move |_| {
            called_clone.store(true, Ordering::SeqCst);
        });
        
        // 取消订阅后，不应该再收到事件
        event_bus.unsubscribe(id);
        event_bus.publish(TestEvent(42));
        assert!(!called.load(Ordering::SeqCst));
    }
    
    #[test]
    fn test_multiple_event_types() {
        let event_bus = EventBus::new();
        let event1_called = Arc::new(AtomicBool::new(false));
        let event1_called_clone = event1_called.clone();
        let event2_called = Arc::new(AtomicBool::new(false));
        let event2_called_clone = event2_called.clone();
        
        #[derive(Clone)]
        struct Event1;
        
        #[derive(Clone)]
        struct Event2;
        
        event_bus.subscribe::<Event1, _>(move |_| {
            event1_called_clone.store(true, Ordering::SeqCst);
        });
        
        event_bus.subscribe::<Event2, _>(move |_| {
            event2_called_clone.store(true, Ordering::SeqCst);
        });
        
        // 发布Event1，只有Event1的监听器应该被触发
        event_bus.publish(Event1);
        assert!(event1_called.load(Ordering::SeqCst));
        assert!(!event2_called.load(Ordering::SeqCst));
        
        // 重置状态
        event1_called.store(false, Ordering::SeqCst);
        
        // 发布Event2，只有Event2的监听器应该被触发
        event_bus.publish(Event2);
        assert!(!event1_called.load(Ordering::SeqCst));
        assert!(event2_called.load(Ordering::SeqCst));
    }
}