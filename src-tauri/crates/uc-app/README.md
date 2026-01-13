# uc-app - Application Layer

## Overview / 概述

The `uc-app` crate contains the application layer logic, coordinating between the core domain models and infrastructure/platform implementations.
`uc-app` crate 包含应用层逻辑，协调核心领域模型与基础设施/平台实现之间的交互。

## Structure / 结构

```
uc-app/
├── deps.rs          # AppDeps struct (dependency injection)
├── state/           # Application state management
├── event/           # Event handling
├── use_cases/       # Use case implementations
└── lib.rs           # Public exports
```

## Dependency Injection / 依赖注入

The `App` struct now supports two construction methods:
`App` 结构现在支持两种构造方法：

1. **`App::new(AppDeps)`** (Preferred) / （推荐）
   - Direct dependency injection / 直接依赖注入
   - Constructor signature = dependency manifest / 构造函数签名即依赖清单
   - No hidden magic / 无隐藏魔法

2. **`AppBuilder::build()`** (Legacy, to be removed)
   - **旧版，将被移除**
   - Kept for backward compatibility during migration / 迁移期间保持向后兼容
   - Will be removed in Phase 4 / 将在 Phase 4 中移除

```rust
// Recommended / 推荐
let app = App::new(AppDeps {
    clipboard: Arc::new(clipboard_impl),
    encryption: Arc::new(encryption_impl),
    // ...
});

// Legacy (will be removed) / 旧版（将被移除）
let app = AppBuilder::new()
    .with_clipboard(clipboard_impl)
    .build()?;
```

## Migration Status / 迁移状态

This crate is undergoing refactoring as part of the bootstrap architecture migration.
此 crate 正在作为引导架构重构的一部分进行重构。

- [x] Phase 1: AppDeps introduced / AppDeps 已引入
- [ ] Phase 2: Bootstrap integration / Bootstrap 集成
- [ ] Phase 3: Event system migration / 事件系统迁移
- [ ] Phase 4: Remove AppBuilder / 移除 AppBuilder
