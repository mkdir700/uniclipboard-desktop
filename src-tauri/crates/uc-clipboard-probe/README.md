# uc-clipboard-probe

UniClipboard 的剪切板探测和快照工具。

## 功能

- **watch**: 监控剪切板变化
- **capture**: 捕获当前剪切板内容到文件
- **restore**: 从文件恢复剪切板内容
- **inspect**: 检查快照文件内容

## 安装和运行

在 UniClipboard 项目根目录下：

```bash
# 构建和运行
cargo run -p uc-clipboard-probe -- --help

# 监控剪切板变化
cargo run -p uc-clipboard-probe -- watch

# 监控最多 10 个事件
cargo run -p uc-clipboard-probe -- watch --max-events 10

# 捕获当前剪切板
cargo run -p uc-clipboard-probe -- capture --out snapshot.json

# 从文件恢复剪切板
cargo run -p uc-clipboard-probe -- restore --in snapshot.json

# 检查快照文件
cargo run -p uc-clipboard-probe -- inspect --in snapshot.json
```

## 依赖

- `uc-core`: UniClipboard 核心类型和端口定义
- `uc-platform`: 平台特定的剪切板实现
- `clipboard-rs`: 跨平台剪切板访问
- `clap`: 命令行参数解析
- `serde_json`: JSON 序列化
- `chrono`: 时间处理

## 架构

这个工具独立于主要的 UniClipboard 应用，可以作为：

1. **调试工具**: 在开发过程中诊断剪切板问题
2. **测试工具**: 自动化测试中验证剪切板行为
3. **独立工具**: 单独使用剪切板快照功能

## 输出格式

快照文件使用 JSON 格式，包含：

```json
{
  "ts_ms": 1768179006974,
  "representations": [
    {
      "format_id": "text",
      "mime": "text/plain",
      "bytes": "base64编码的内容"
    }
  ]
}
```

## 许可证

与 UniClipboard 项目相同的许可证。
