# AppDeps - Dependency Grouping

## Purpose / 目的

Groups all application dependencies into a single struct for clean dependency injection.
将所有应用依赖分组到单个结构体中，以实现干净的依赖注入。

## Important / 重要

**This is NOT a Builder pattern.**
**这不是 Builder 模式。**

- ❌ No build steps / ❌ 无构建步骤
- ❌ No default values / ❌ 无默认值
- ❌ No hidden logic / ❌ 无隐藏逻辑
- ✅ Just parameter grouping / ✅ 仅参数打包

## Constructor Signature = Dependency Manifest / 构造函数签名即依赖清单

```rust
pub fn new(deps: AppDeps) -> App
```

Looking at this function signature tells you ALL dependencies of App.
查看此函数签名即可知道 App 的所有依赖。

No hidden dependencies, no defaults, no magic.
无隐藏依赖，无默认值，无魔法。

## Migration Path / 迁移路径

- Phase 1: AppDeps added alongside AppBuilder (compatibility) / AppDeps 与 AppBuilder 共存（兼容）
- Phase 2: Bootstrap starts using App::new(AppDeps) / Bootstrap 开始使用 App::new(AppDeps)
- Phase 4: AppBuilder removed / 移除 AppBuilder
