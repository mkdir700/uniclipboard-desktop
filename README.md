## 📝 项目介绍

[English](./README_EN.md) | 简体中文

UniClipboard 是一个剪切板同步工具，旨在为用户提供无缝的剪切板共享体验，让您在不同设备间即时共享文本、图片和文件。

![Image](./assets/demo.png)

<div align="center">

  <br/>

  <a href="https://github.com/UniClipboard/UniClipboard/releases">
    <img
      alt="Windows"
      src="https://img.shields.io/badge/-Windows-blue?style=flat-square&logo=data:image/svg+xml;base64,PHN2ZyB0PSIxNzI2MzA1OTcxMDA2IiBjbGFzcz0iaWNvbiIgdmlld0JveD0iMCAwIDEwMjQgMTAyNCIgdmVyc2lvbj0iMS4xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHAtaWQ9IjE1NDgiIHdpZHRoPSIxMjgiIGhlaWdodD0iMTI4Ij48cGF0aCBkPSJNNTI3LjI3NTU1MTYxIDk2Ljk3MTAzMDEzdjM3My45OTIxMDY2N2g0OTQuNTEzNjE5NzVWMTUuMDI2NzU3NTN6TTUyNy4yNzU1NTE2MSA5MjguMzIzNTA4MTVsNDk0LjUxMzYxOTc1IDgwLjUyMDI4MDQ5di00NTUuNjc3NDcxNjFoLTQ5NC41MTM2MTk3NXpNNC42NzA0NTEzNiA0NzAuODMzNjgyOTdINDIyLjY3Njg1OTI1VjExMC41NjM2ODE5N2wtNDE4LjAwNjQwNzg5IDY5LjI1Nzc5NzUzek00LjY3MDQ1MTM2IDg0Ni43Njc1OTcwM0w0MjIuNjc2ODU5MjUgOTE0Ljg2MDMxMDEzVjU1My4xNjYzMTcwM0g0LjY3MDQ1MTM2eiIgcC1pZD0iMTU0OSIgZmlsbD0iI2ZmZmZmZiI+PC9wYXRoPjwvc3ZnPg=="
    />
  </a >  
  <a href="https://github.com/UniClipboard/UniClipboard/releases">
    <img
      alt="MacOS"
      src="https://img.shields.io/badge/-MacOS-black?style=flat-square&logo=apple&logoColor=white"
    />
  </a >
  <a href="https://github.com/UniClipboard/UniClipboard/releases">
    <img
      alt="Linux"
      src="https://img.shields.io/badge/-Linux-purple?style=flat-square&logo=linux&logoColor=white"
    />
  </a>

  <div>
    <a href="./LICENSE">
      <img
        src="https://img.shields.io/github/license/UniClipboard/UniClipboard?style=flat-square"
      />
    </a >
    <a href="https://github.com/UniClipboard/UniClipboard/releases">
      <img
        src="https://img.shields.io/github/v/release/UniClipboard/UniClipboard?include_prereleases&style=flat-square"
      />
    </a >
    <a href="https://codecov.io/gh/UniClipboard/UniClipboard" >
      <img src="https://codecov.io/gh/UniClipboard/UniClipboard/branch/main/graph/badge.svg?token=QZfjXOsQTp"/>
    </a>
  </div>

</div>

> [!WARNING]
> UniClipboard 目前处于积极开发阶段，可能存在功能不稳定或缺失的情况。欢迎体验并提供反馈！

## ✨ 功能特点

- **跨平台支持**: 支持 Windows、macOS 和 Linux 操作系统
- **实时同步**: 在连接的设备间即时共享剪切板内容
- **丰富内容类型**: 支持文本、图片、文件等多种内容类型
- **安全加密**: 使用 XChaCha20-Poly1305 AEAD 加密算法确保数据传输安全
- **多设备管理**: 便捷添加和管理多台设备
- **灵活配置**: 提供丰富的自定义设置选项

## 🚀 安装方法

### 从 Releases 下载

访问 [GitHub Releases](https://github.com/UniClipboard/UniClipboard/releases) 页面，下载适合您操作系统的安装包。

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/UniClipboard/UniClipboard.git
cd uniclipboard-desktop

# 安装依赖
bun install

# 开发模式启动
bun tauri dev

# 构建应用
bun tauri build
```

## 🎮 使用说明

1. **首次启动**: 启动应用后，进行基本设置并创建您的设备身份
2. **添加设备**: 在"设备"页面中，点击"添加设备"按钮添加新设备
3. **剪切板同步**: 复制内容后，它将自动同步到所有已连接的设备
4. **设置**: 在"设置"页面自定义应用行为、网络和安全选项

### 主要页面

- **仪表盘**: 概览当前剪切板状态和设备连接情况
- **设备**: 管理和配对设备，设置设备访问权限
- **设置**: 配置应用参数，包括通用设置、同步选项、安全与隐私、网络设置和存储管理

## 🔧 高级功能

### 网络配置

支持多种网络连接模式，可根据您的网络环境进行配置：

- **局域网同步**: 默认使用局域网直接同步
- **WebDAV 同步**: 开发中

### 安全功能

- **端到端加密**: 所有设备间传输的数据都经过加密保护
- **XChaCha20-Poly1305 加密**: 使用现代 AEAD 加密算法提供认证加密
  - 24 字节随机 nonce，有效降低 nonce 重用风险
  - 32 字节（256 位）加密密钥
  - 提供密文完整性和真实性验证
- **Argon2id 密钥派生**: 从用户密码安全派生加密密钥
  - 内存成本：128 MB
  - 迭代次数：3 次
  - 并行度：4 线程
  - 抗 GPU/ASIC 破解攻击
- **密钥管理**: 分层密钥架构保护数据安全
  - 主密钥（MasterKey）用于剪贴板内容加密
  - 密钥加密密钥（KEK）通过 Argon2id 从密码派生
  - KEK 安全存储于系统密钥环（macOS Keychain、Windows Credential Manager、Linux Secret Service）
  - 主密钥加密存储于 KeySlot 文件
- **设备授权**: 精确控制每台设备的访问权限

## 🤝 参与贡献

非常欢迎各种形式的贡献！如果您对改进 UniClipboard 感兴趣，请：

1. Fork 本仓库
2. 创建您的特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交您的更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建一个 Pull Request

## 📄 许可证

本项目采用 Apache-2.0 许可证 - 详情请参阅 [LICENSE](./LICENSE) 文件。

## 🙏 鸣谢

- [Tauri](https://tauri.app) - 提供跨平台应用框架
- [React](https://react.dev) - 前端界面开发框架
- [Rust](https://www.rust-lang.org) - 安全高效的后端实现语言

---

💡 **有问题或建议?** [创建 Issue](https://github.com/UniClipboard/UniClipboard/issues/new) 或联系我们讨论!
