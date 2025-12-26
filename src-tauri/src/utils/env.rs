use std::env;

/// 判断是否为开发环境
pub fn is_development() -> bool {
    // 通过环境变量或编译时特性来判断
    // 1. 优先检查环境变量 UNICLIPBOARD_ENV
    if let Ok(env_val) = env::var("UNICLIPBOARD_ENV") {
        return env_val == "development";
    }
    // 2. 检查 debug_assertions 是否启用（编译时特性）
    #[cfg(debug_assertions)]
    {
        return true;
    }
    #[cfg(not(debug_assertions))]
    {
        return false;
    }
}
