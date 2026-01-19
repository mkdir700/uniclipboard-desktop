# Splashscreen 生成器设计

**Date:** 2026-01-18
**Status:** Design Approved
**Related:** [2026-01-18-loadingscreen-redesign.md](./2026-01-18-loadingscreen-redesign.md)

## 概述

将 `LoadingScreen.tsx` 转换为构建工具，生成自包含的 `splashscreen.html`（内联所有 CSS/JS），支持通过 CSS 变量切换多主题。

## 背景

### 现状

- `LoadingScreen.tsx`: React 组件，使用 Tailwind CSS 和 i18n
- `splashscreen.html`: 独立 HTML 文件，在应用加载前显示
- 两者样式不统一，维护困难

### 目标

- 复用 `LoadingScreen.tsx` 的设计语言
- 生成完全自包含的 HTML（无外部依赖）
- 支持所有应用主题（zinc、catppuccin、claude、t3chat）

## 主题系统分析

应用使用 Shadcn UI 的 CSS 变量主题系统：

```css
[data-theme='zinc']:not(.dark) { --background: oklch(1 0 0); ... }
[data-theme='zinc'].dark { --background: oklch(0.21 0.006 285.885); ... }
[data-theme='catppuccin']:not(.dark) { ... }
[data-theme='catppuccin'].dark { ... }
```

- **4 个主题**: zinc、catppuccin、claude、t3chat
- **2 个变体**: light (`.dark` class 不存在)、dark (`.dark` class 存在)
- **主题保存**: 在后端（Tauri 配置），通过 `get_settings` / `update_settings` 命令

## 架构设计

### 文件结构

```
新增文件：
├── src/splashscreen/
│   ├── template.html       # HTML 模板（包含占位符）
│   └── animations.css      # 动画 CSS 定义
├── scripts/
│   └── generate-splash.ts  # 构建脚本
└── public/splashscreen.html # 生成产物（自动）

修改文件：
├── src/contexts/SettingContext.tsx  # 添加 localStorage 同步（可选）
└── package.json                     # 添加 scripts 和依赖

删除文件：
└── src/components/LoadingScreen.tsx # 不再需要
```

### 构建流程

```
1. 读取 src/styles/themes/*.css
   ↓
2. 提取 CSS 变量，移除 @layer base 包装
   ↓
3. 读取 src/splashscreen/template.html
   ↓
4. 注入主题 CSS + 动画 CSS
   ↓
5. 写入 public/splashscreen.html
```

### 组件设计

#### 1. 构建脚本 (`scripts/generate-splash.ts`)

```typescript
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'
import glob from 'glob'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(__dirname, '..')

async function generateSplashscreen() {
  // 1. 读取并合并所有主题 CSS
  const themeFiles = await glob('src/styles/themes/*.css', { cwd: root })
  const themesCss = themeFiles
    .map(file => {
      const content = fs.readFileSync(path.join(root, file), 'utf-8')
      // 移除 @layer base 包装和闭合括号
      return content.replace(/@layer base\s*{?\s*/g, '').replace(/}\s*$/, '')
    })
    .join('\n')

  // 2. 读取动画 CSS
  const animationsCss = fs.readFileSync(path.join(root, 'src/splashscreen/animations.css'), 'utf-8')

  // 3. 读取 HTML 模板
  const template = fs.readFileSync(path.join(root, 'src/splashscreen/template.html'), 'utf-8')

  // 4. 注入内容
  const html = template
    .replace('/* ANIMATIONS_CSS */', animationsCss)
    .replace('/* THEMES_CSS */', themesCss)

  // 5. 写入输出
  fs.writeFileSync(path.join(root, 'public/splashscreen.html'), html)

  console.log('✓ Generated public/splashscreen.html')
}

generateSplashscreen().catch(console.error)
```

#### 2. HTML 模板 (`src/splashscreen/template.html`)

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>UniClipboard</title>
    <style>
      /* ========== ANIMATIONS_CSS ========== */
      /* 由构建脚本注入 animations.css 内容 */

      /* ========== 基础布局 ========== */
      body {
        margin: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 100vh;
        background: var(--background);
        color: var(--foreground);
      }

      .splash-root {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 2rem;
      }

      .splash-brand {
        font-size: 2.5rem;
        font-weight: 300;
        letter-spacing: 0.1em;
        color: var(--foreground);
      }

      .splash-letter {
        display: inline-block;
        animation: fade-in-up 0.6s ease-out backwards;
      }

      .splash-status {
        font-size: 0.875rem;
        color: var(--muted-foreground);
        font-weight: 500;
      }

      .splash-dots {
        display: flex;
        gap: 0.5rem;
      }

      .splash-dot {
        width: 0.5rem;
        height: 0.5rem;
        border-radius: 50%;
        background: var(--primary);
        opacity: 0.7;
        animation: pulse-wave 1.5s ease-in-out infinite;
      }

      /* ========== THEMES_CSS ========== */
      /* 由构建脚本注入所有主题 CSS */

      /* ========== 默认降级样式 ========== */
      :root {
        --background: oklch(1 0 0);
        --foreground: oklch(0.21 0.006 285.885);
        --muted-foreground: oklch(0.552 0.014 285.938);
        --primary: oklch(0.21 0.006 285.885);
      }
    </style>
  </head>
  <body>
    <div class="splash-root">
      <div class="splash-brand">
        <span class="splash-letter" style="animation-delay: 0s">U</span>
        <span class="splash-letter" style="animation-delay: 0.05s">n</span>
        <span class="splash-letter" style="animation-delay: 0.1s">i</span>
        <span class="splash-letter" style="animation-delay: 0.15s">C</span>
        <span class="splash-letter" style="animation-delay: 0.2s">l</span>
        <span class="splash-letter" style="animation-delay: 0.25s">i</span>
        <span class="splash-letter" style="animation-delay: 0.3s">p</span>
        <span class="splash-letter" style="animation-delay: 0.35s">b</span>
        <span class="splash-letter" style="animation-delay: 0.4s">o</span>
        <span class="splash-letter" style="animation-delay: 0.45s">a</span>
        <span class="splash-letter" style="animation-delay: 0.5s">r</span>
        <span class="splash-letter" style="animation-delay: 0.55s">d</span>
      </div>
      <div class="splash-status">Initializing...</div>
      <div class="splash-dots">
        <span class="splash-dot" style="animation-delay: -0.4s"></span>
        <span class="splash-dot" style="animation-delay: -0.2s"></span>
        <span class="splash-dot" style="animation-delay: 0s"></span>
      </div>
    </div>

    <script>
      ;(async function () {
        const root = document.documentElement
        let theme = 'zinc' // 默认主题
        let mode = null // null = 未设置

        // ========== 1. 尝试 Tauri API ==========
        if (window.__TAURI__) {
          try {
            const { invoke } = await import('@tauri-apps/api/core')
            const settings = await invoke('get_settings')

            const userTheme = settings?.general?.theme
            const userThemeColor = settings?.general?.theme_color || 'zinc'

            theme = userThemeColor

            if (userTheme === 'light') {
              mode = 'light'
            } else if (userTheme === 'dark') {
              mode = 'dark'
            }
          } catch (e) {
            console.warn('无法读取 Tauri 配置:', e)
          }
        }

        // ========== 2. 回退到 localStorage ==========
        if (mode === null) {
          const savedTheme = localStorage.getItem('uc-theme')
          const savedThemeColor = localStorage.getItem('uc-theme-color')

          if (savedThemeColor) theme = savedThemeColor
          if (savedTheme === 'light' || savedTheme === 'dark') {
            mode = savedTheme
          }
        }

        // ========== 3. 系统默认 ==========
        if (mode === null) {
          mode = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
        }

        // ========== 应用主题 ==========
        root.setAttribute('data-theme', theme)
        if (mode === 'dark') {
          root.classList.add('dark')
        }

        // ========== 监听系统主题变化 ==========
        window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', e => {
          if (!localStorage.getItem('uc-theme')) {
            root.classList.toggle('dark', e.matches)
          }
        })
      })()
    </script>
  </body>
</html>
```

#### 3. 动画 CSS (`src/splashscreen/animations.css`)

```css
@keyframes fade-in-up {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes pulse-wave {
  0%,
  100% {
    opacity: 0.4;
    transform: scale(1);
  }
  50% {
    opacity: 1;
    transform: scale(1.2);
  }
}
```

### 可选：主应用 localStorage 同步

在 `src/contexts/SettingContext.tsx` 的 `applyTheme` 函数中添加：

```typescript
// 同步到 localStorage（供 splashscreen 使用）
if (theme !== 'system') {
  localStorage.setItem('uc-theme', theme)
} else {
  localStorage.removeItem('uc-theme')
}
localStorage.setItem('uc-theme-color', themeColor)
```

## 数据流

```
用户启动应用
    ↓
splashscreen.html 显示
    ↓
JavaScript 检测主题：
  1. Tauri API → get_settings
  2. localStorage → uc-theme, uc-theme-color
  3. 系统默认 → prefers-color-scheme
    ↓
应用主题 (data-theme + .dark class)
    ↓
主应用加载
    ↓
splashscreen 隐藏
```

## 错误处理

| 场景                | 处理方式                      |
| ------------------- | ----------------------------- |
| Tauri API 未就绪    | 捕获异常，回退到 localStorage |
| 配置文件损坏        | 使用默认主题（zinc + system） |
| 主题文件缺失        | 构建脚本报错，不生成 HTML     |
| JavaScript 执行失败 | 降级到 CSS 默认值             |

## 构建集成

在 `package.json` 添加：

```json
{
  "scripts": {
    "generate:splash": "bun run scripts/generate-splash.ts",
    "prebuild": "bun run generate:splash",
    "predev": "bun run generate:splash"
  },
  "devDependencies": {
    "glob": "^10.3.0"
  }
}
```

这样在开发和构建前都会自动生成最新的 splashscreen。

## 测试清单

- [ ] 所有 4 个主题（zinc、catppuccin、claude、t3chat）正确显示
- [ ] 每个主题的 light/dark 模式正确切换
- [ ] 动画流畅（字母淡入、点脉冲）
- [ ] 系统主题变化时实时响应
- [ ] Tauri 配置读取正确
- [ ] Tauri API 失败时正确降级
- [ ] 文件大小合理（< 50 KB）

## 实施步骤

1. 创建 `src/splashscreen/` 目录和模板文件
2. 编写构建脚本 `scripts/generate-splash.ts`
3. 测试生成 HTML 是否正确
4. （可选）添加 localStorage 同步到 `SettingContext.tsx`
5. 删除 `LoadingScreen.tsx`
6. 更新 `App.tsx` 移除 LoadingScreen 引用
7. 运行测试清单验证

## 后续优化

- [ ] 添加构建时 CSS 压缩
- [ ] 支持自定义品牌名称
- [ ] 添加加载进度条（从主应用接收事件）
