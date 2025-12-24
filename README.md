![uniclipboard-desktop](https://socialify.git.ci/mkdir700/uniclipboard-desktop/image?description=1&descriptionEditable=%E4%B8%80%E4%B8%AA%E8%B7%A8%E5%B9%B3%E5%8F%B0%E5%89%AA%E5%88%87%E6%9D%BF%E5%85%B1%E4%BA%AB%E5%B7%A5%E5%85%B7%EF%BC%8C%E6%97%A8%E5%9C%A8%E6%89%93%E9%80%A0%E6%97%A0%E7%BC%9D%E7%9A%84%E5%89%AA%E5%88%87%E6%9D%BF%E4%BD%93%E9%AA%8C&font=Raleway&language=1&name=1&owner=1&pattern=Circuit%20Board&theme=Auto)

<div align="center">
  <br/>
    
  <a href="https://github.com/mkdir700/uniclipboard-desktop/releases">
    <img
      alt="Windows"
      src="https://img.shields.io/badge/-Windows-blue?style=flat-square&logo=data:image/svg+xml;base64,PHN2ZyB0PSIxNzI2MzA1OTcxMDA2IiBjbGFzcz0iaWNvbiIgdmlld0JveD0iMCAwIDEwMjQgMTAyNCIgdmVyc2lvbj0iMS4xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHAtaWQ9IjE1NDgiIHdpZHRoPSIxMjgiIGhlaWdodD0iMTI4Ij48cGF0aCBkPSJNNTI3LjI3NTU1MTYxIDk2Ljk3MTAzMDEzdjM3My45OTIxMDY2N2g0OTQuNTEzNjE5NzVWMTUuMDI2NzU3NTN6TTUyNy4yNzU1NTE2MSA5MjguMzIzNTA4MTVsNDk0LjUxMzYxOTc1IDgwLjUyMDI4MDQ5di00NTUuNjc3NDcxNjFoLTQ5NC41MTM2MTk3NXpNNC42NzA0NTEzNiA0NzAuODMzNjgyOTdINDIyLjY3Njg1OTI1VjExMC41NjM2ODE5N2wtNDE4LjAwNjQwNzg5IDY5LjI1Nzc5NzUzek00LjY3MDQ1MTM2IDg0Ni43Njc1OTcwM0w0MjIuNjc2ODU5MjUgOTE0Ljg2MDMxMDEzVjU1My4xNjYzMTcwM0g0LjY3MDQ1MTM2eiIgcC1pZD0iMTU0OSIgZmlsbD0iI2ZmZmZmZiI+PC9wYXRoPjwvc3ZnPg=="
    />
  </a >  
  <a href="https://github.com/mkdir700/uniclipboard-desktop/releases">
    <img
      alt="MacOS"
      src="https://img.shields.io/badge/-MacOS-black?style=flat-square&logo=apple&logoColor=white"
    />
  </a >
  <a href="https://github.com/mkdir700/uniclipboard-desktop/releases">
    <img 
      alt="Linux"
      src="https://img.shields.io/badge/-Linux-purple?style=flat-square&logo=linux&logoColor=white" 
    />
  </a>

  <div>
    <a href="./LICENSE">
      <img
        src="https://img.shields.io/github/license/mkdir700/uniclipboard-desktop?style=flat-square"
      />
    </a >
    <a href="https://github.com/mkdir700/uniclipboard-desktop/releases">
      <img
        src="https://img.shields.io/github/v/release/mkdir700/uniclipboard-desktop?include_prereleases&style=flat-square"
      />
    </a >
    <a href="https://codecov.io/gh/mkdir700/uniclipboard-desktop">
      <img src="https://img.shields.io/codecov/c/github/mkdir700/uniclipboard-desktop/master?style=flat-square" />
    </a>
    <a href="https://github.com/mkdir700/uniclipboard-desktop/releases">
      <img
        src="https://img.shields.io/github/downloads/mkdir700/uniclipboard-desktop/total?style=flat-square"
      />  
    </a >
  </div>

</div>

> [!WARNING]
> uniclipboard-desktop 目前处于积极开发阶段，可能存在功能不稳定或缺失的情况。欢迎体验并提供反馈！

## 📝 项目介绍

uniclipboard-desktop 是一个功能强大的跨平台剪切板同步工具，旨在为用户提供无缝的剪切板共享体验。无论您使用的是 Windows、macOS 还是 Linux，uniclipboard-desktop 都能让您在不同设备间即时共享文本、图片和文件，提升工作效率。

![Image](https://github.com/user-attachments/assets/6bc63e44-d11c-4675-9f4c-c8c8368453a0)

## ✨ 功能特点

- 🌐 **跨平台支持**: 支持 Windows、macOS 和 Linux 操作系统
- 🔄 **实时同步**: 在连接的设备间即时共享剪切板内容
- 📊 **丰富内容类型**: 支持文本、图片、文件等多种内容类型
- 🔐 **安全加密**: 使用 AES-GCM 加密算法确保数据传输安全
- 📱 **多设备管理**: 便捷添加和管理多台设备
- ⚙️ **灵活配置**: 提供丰富的自定义设置选项

## 🚀 安装方法

### 从 Releases 下载

访问 [GitHub Releases](https://github.com/mkdir700/uniclipboard-desktop/releases) 页面，下载适合您操作系统的安装包。

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/mkdir700/uniclipboard-desktop.git
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

uniclipboard-desktop 支持多种网络连接模式，可根据您的网络环境进行配置：

- **局域网同步**: 默认使用局域网直接同步
- **WebDAV 同步**: 支持通过 WebDAV 服务器同步数据

### 安全功能

- **端到端加密**: 所有设备间传输的数据都经过加密保护
- **设备授权**: 精确控制每台设备的访问权限

## 🤝 参与贡献

非常欢迎各种形式的贡献！如果您对改进 uniclipboard-desktop 感兴趣，请：

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

💡 **有问题或建议?** [创建 Issue](https://github.com/mkdir700/uniclipboard-desktop/issues/new) 或联系我们讨论!