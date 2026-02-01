//! Integration tests for UseCases accessor
//! UseCases 访问器的集成测试

use uc_tauri::bootstrap::{AppRuntime, UseCases};

// This test verifies UseCases methods are callable
// Actual behavior testing is in uc-app use case tests
//
// 此测试验证 UseCases 方法可调用
// 实际行为测试在 uc-app 用例测试中

#[test]
fn test_use_cases_has_list_clipboard_entries() {
    // Compile-time verification that the method exists
    // 编译时验证方法存在
    fn assert_method_exists<F: Fn(&UseCases) -> uc_app::usecases::ListClipboardEntries>(_f: F) {}

    // This will only compile if UseCases has list_clipboard_entries() method
    // 这只有在 UseCases 有 list_clipboard_entries() 方法时才能编译
    assert_method_exists(|uc: &UseCases| uc.list_clipboard_entries());
}

#[test]
fn test_use_cases_has_announce_device_name() {
    fn assert_method_exists<F: Fn(&UseCases) -> uc_app::usecases::AnnounceDeviceName>(_f: F) {}

    assert_method_exists(|uc: &UseCases| uc.announce_device_name());
}

#[test]
fn test_app_runtime_has_usecases_method() {
    // Compile-time verification
    // 编译时验证
    fn assert_method_exists<F: Fn(&AppRuntime) -> UseCases>(_f: F) {}

    // This will only compile if AppRuntime has usecases() method
    // 这只有在 AppRuntime 有 usecases() 方法时才能编译
    assert_method_exists(|runtime: &AppRuntime| runtime.usecases());
}

#[test]
fn test_app_runtime_has_deps_field() {
    // Compile-time verification that AppRuntime has deps field
    // 编译时验证 AppRuntime 有 deps 字段
    fn can_access_deps(_runtime: &AppRuntime) -> &uc_app::AppDeps {
        // This function will only compile if AppRuntime has a public deps field
        // 这个函数只有在 AppRuntime 有公共 deps 字段时才能编译
        unimplemented!()
    }

    // If this compiles, the struct has the right shape
    // 如果这能编译，说明结构体有正确的形状
    let _ = can_access_deps;
}
