use std::sync::Arc;

use uniclipboard::key_mouse_monitor::{KeyMouseMonitor, KeyMouseMonitorTrait};
use tokio::time::{sleep, Duration, Instant};
use serial_test::serial;
use enigo::Direction::Press;
use enigo::{Coordinate, Enigo, Settings, Mouse, Keyboard, Key};

#[tokio::test]
#[cfg_attr(not(feature = "hardware_tests"), ignore)]
#[serial]
async fn test_get_last_activity() {
    let monitor = KeyMouseMonitor::new(Duration::from_secs(5));
    let last_activity = monitor.get_last_activity().await;
    assert!(Instant::now().duration_since(last_activity) < Duration::from_secs(1));
}

#[tokio::test]
#[cfg_attr(not(feature = "hardware_tests"), ignore)]
#[serial]
async fn test_update_last_activity() {
    let monitor = KeyMouseMonitor::new(Duration::from_secs(5));
    let initial_activity = monitor.get_last_activity().await;
    sleep(Duration::from_millis(100)).await;
    monitor.update_last_activity().await;
    let updated_activity = monitor.get_last_activity().await;
    assert!(updated_activity > initial_activity);
}

#[tokio::test]
#[cfg_attr(not(feature = "hardware_tests"), ignore)]
#[serial]
async fn test_is_sleep() {
    let monitor = KeyMouseMonitor::new(Duration::from_millis(100));
    assert!(!monitor.is_sleep().await);
    sleep(Duration::from_millis(150)).await;
    assert!(monitor.is_sleep().await);
}

#[tokio::test]
#[cfg_attr(not(feature = "hardware_tests"), ignore)]
#[serial]
async fn test_concurrent_updates() {
    let monitor = Arc::new(KeyMouseMonitor::new(Duration::from_secs(5)));
    let monitor_clone = Arc::clone(&monitor);

    let task1 = tokio::spawn(async move {
        for _ in 0..100 {
            monitor.update_last_activity().await;
            sleep(Duration::from_millis(1)).await;
        }
    });

    let task2 = tokio::spawn(async move {
        for _ in 0..100 {
            monitor_clone.get_last_activity().await;
            sleep(Duration::from_millis(1)).await;
        }
    });

    let _ = tokio::join!(task1, task2);
}

/// 测试鼠标移动检测
/// 
/// !测试该用例需要硬件支持，请在硬件上运行
/// 测试该用例时，请确保没有其他鼠标移动，否则会导致测试失败
#[tokio::test]
#[cfg_attr(not(feature = "hardware_tests"), ignore)]
#[serial]
async fn test_mouse_movement_detection() {
    let monitor = KeyMouseMonitor::new(Duration::from_secs(1));
    assert_eq!(*monitor.is_running.lock().await, false); 
    monitor.start().await;
    assert_eq!(*monitor.is_running.lock().await, true);

    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // 等待一小段时间, 超过设置的睡眠时间
    sleep(Duration::from_secs(3)).await;

    assert!(monitor.is_sleep().await);

    // 使用 enigo 移动鼠标
    enigo.move_mouse(10, 10, Coordinate::Rel).unwrap();

    // 等待一小段时间，确保活动被检测到
    sleep(Duration::from_millis(500)).await;

    // 测试 is_sleep 方法
    assert!(!monitor.is_sleep().await);
}

/// 测试键盘检测
/// 
/// !测试该用例需要硬件支持，请在硬件上运行
/// 测试该用例时，请确保没有其他键盘操作，否则会导致测试失败
#[tokio::test]
#[cfg_attr(not(feature = "hardware_tests"), ignore)]
#[serial]
async fn test_keyboard_detection() {
    let monitor = KeyMouseMonitor::new(Duration::from_secs(1));
    assert_eq!(*monitor.is_running.lock().await, false); 
    monitor.start().await;
    assert_eq!(*monitor.is_running.lock().await, true);

    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // 等待一小段时间, 超过设置的睡眠时间
    sleep(Duration::from_secs(3)).await;

    assert!(monitor.is_sleep().await);

    // 使用 enigo 按下键盘 ESC 键
    enigo.key(Key::Escape, Press).unwrap();
    sleep(Duration::from_millis(300)).await;

    // 等待一小段时间，确保活动被检测到
    sleep(Duration::from_millis(500)).await;

    // 测试 is_sleep 方法
    assert!(!monitor.is_sleep().await);
}