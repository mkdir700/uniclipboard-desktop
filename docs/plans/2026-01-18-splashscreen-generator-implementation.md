# Splashscreen 生成器实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将 `LoadingScreen.tsx` React 组件转换为构建时生成的自包含 `splashscreen.html`，支持通过 CSS 变量切换 4 种主题（zinc、catppuccin、claude、t3chat）的 light/dark 模式。

**Architecture:**

- 构建时脚本 (`scripts/generate-splash.ts`) 读取主题 CSS 文件并注入到 HTML 模板
- 生成的 `splashscreen.html` 内联所有 CSS/JS，无外部依赖
- JavaScript 在运行时检测主题（优先级：Tauri API > localStorage > 系统默认）

**Tech Stack:**

- TypeScript (Node.js) 构建脚本
- 原生 ES Modules (`import.meta.url`)
- glob 库用于文件匹配
- CSS 变量主题系统

---

## 前置准备

### Task 0: 安装 glob 依赖

**Files:**

- Modify: `package.json`

**Step 1: 添加 glob 到 devDependencies**

编辑 `package.json`，在 `devDependencies` 中添加：

```json
"glob": "^10.3.0"
```

**Step 2: 安装依赖**

Run: `bun install`
Expected: glob 包被安装到 `node_modules`

**Step 3: 验证安装**

Run: `grep '"glob"' package.json`
Expected: 输出包含 `"glob": "^10.3.0"`

**Step 4: Commit**

```bash
git add package.json bun.lockb
git commit -m "build: add glob dependency for splashscreen generator"
```

---

## 构建脚本

### Task 1: 创建构建脚本骨架

**Files:**

- Create: `scripts/generate-splash.ts`

**Step 1: 创建基础脚本结构**

创建 `scripts/generate-splash.ts`：

```typescript
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(__dirname, '..')

async function generateSplashscreen() {
  console.log('Generating splashscreen.html...')
}

generateSplashscreen().catch(console.error)
```

**Step 2: 验证脚本可运行**

Run: `bun run scripts/generate-splash.ts`
Expected: 输出 "Generating splashscreen.html..."

**Step 3: Commit**

```bash
git add scripts/generate-splash.ts
git commit -m "build: add splashscreen generator skeleton"
```

---

### Task 2: 实现主题 CSS 读取和合并

**Files:**

- Modify: `scripts/generate-splash.ts`

**Step 1: 导入 glob 并读取主题文件**

修改 `scripts/generate-splash.ts`：

```typescript
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'
import { glob } from 'glob'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(__dirname, '..')

async function generateSplashscreen() {
  console.log('Generating splashscreen.html...')

  // 1. 读取并合并所有主题 CSS
  const themeFiles = await glob('src/styles/themes/*.css', { cwd: root })
  console.log(`Found ${themeFiles.length} theme files:`, themeFiles)

  const themesCss = themeFiles
    .map(file => {
      const content = fs.readFileSync(path.join(root, file), 'utf-8')
      // 移除 @layer base 包装和闭合括号
      return content.replace(/@layer base\s*{?\s*/g, '').replace(/}\s*$/, '')
    })
    .join('\n')

  console.log(`Merged CSS length: ${themesCss.length} bytes`)

  // 2. 输出到临时文件验证
  fs.writeFileSync(path.join(root, 'public/themes-debug.css'), themesCss)
  console.log('✓ Wrote public/themes-debug.css for verification')
}

generateSplashscreen().catch(console.error)
```

**Step 2: 运行脚本验证 CSS 读取**

Run: `bun run scripts/generate-splash.ts`
Expected: 输出 "Found 4 theme files..." 和 "Merged CSS length: XXX bytes"

**Step 3: 验证输出文件**

Run: `ls -lh public/themes-debug.css`
Expected: 文件存在，大小约 2-3 KB

**Step 4: 检查内容是否正确**

Run: `head -20 public/themes-debug.css`
Expected: 输出 CSS 变量定义（如 `--background: oklch(...)`），无 `@layer base`

**Step 5: 清理临时文件**

Run: `rm public/themes-debug.css`

**Step 6: Commit**

```bash
git add scripts/generate-splash.ts
git commit -m "build: implement theme CSS reading and merging"
```

---

### Task 3: 创建动画 CSS 文件

**Files:**

- Create: `src/splashscreen/animations.css`

**Step 1: 创建动画定义文件**

创建 `src/splashscreen/animations.css`：

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

**Step 2: 验证文件存在**

Run: `ls -lh src/splashscreen/animations.css`
Expected: 文件存在，大小约 200 bytes

**Step 3: Commit**

```bash
git add src/splashscreen/animations.css
git commit -m "feat: add splashscreen animations"
```

---

### Task 4: 创建 HTML 模板

**Files:**

- Create: `src/splashscreen/template.html`

**Step 1: 创建包含占位符的模板**

创建 `src/splashscreen/template.html`：

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

**Step 2: 验证模板文件存在**

Run: `ls -lh src/splashscreen/template.html`
Expected: 文件存在

**Step 3: Commit**

```bash
git add src/splashscreen/template.html
git commit -m "feat: add splashscreen HTML template"
```

---

### Task 5: 完善构建脚本注入逻辑

**Files:**

- Modify: `scripts/generate-splash.ts`

**Step 1: 实现完整注入逻辑**

修改 `scripts/generate-splash.ts`：

```typescript
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'
import { glob } from 'glob'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(__dirname, '..')

async function generateSplashscreen() {
  console.log('Generating splashscreen.html...')

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

**Step 2: 运行生成脚本**

Run: `bun run scripts/generate-splash.ts`
Expected: 输出 "✓ Generated public/splashscreen.html"

**Step 3: 验证生成文件存在**

Run: `ls -lh public/splashscreen.html`
Expected: 文件存在，大小约 8-12 KB

**Step 4: 验证 CSS 已注入**

Run: `grep -c "fade-in-up" public/splashscreen.html`
Expected: 输出 `2`（keyframe 定义和使用）

**Step 5: 验证主题 CSS 已注入**

Run: `grep -c "data-theme='zinc'" public/splashscreen.html`
Expected: 输出 `2`（light 和 dark 模式）

**Step 6: Commit**

```bash
git add scripts/generate-splash.ts public/splashscreen.html
git commit -m "build: implement full splashscreen generation"
```

---

## 集成到构建流程

### Task 6: 添加 npm scripts

**Files:**

- Modify: `package.json`

**Step 1: 添加生成和钩子脚本**

编辑 `package.json`，在 `scripts` 中添加：

```json
"generate:splash": "bun run scripts/generate-splash.ts",
"predev": "bun run generate:splash",
"prebuild": "bun run generate:splash"
```

**Step 2: 验证脚本已添加**

Run: `grep "generate:splash" package.json`
Expected: 输出包含三行 `"generate:splash"` 相关脚本

**Step 3: 测试 predev 钩子**

Run: `bun run dev` （立即 Ctrl+C 停止）
Expected: 先输出 "Generating splashscreen.html..."，然后 "✓ Generated public/splashscreen.html"

**Step 4: 验证文件在 predev 后更新**

Run: `ls -lh public/splashscreen.html`
Expected: 文件存在

**Step 5: Commit**

```bash
git add package.json
git commit -m "build: add splashscreen generation to predev/prebuild hooks"
```

---

## 主应用集成

### Task 7: 添加 localStorage 同步到 SettingContext

**Files:**

- Modify: `src/contexts/SettingContext.tsx`

**Step 1: 修改 applyTheme 函数添加 localStorage 同步**

编辑 `src/contexts/SettingContext.tsx`，找到 `applyTheme` 函数（约 163 行），在函数末尾添加：

```typescript
const applyTheme = () => {
  const theme = setting?.general.theme
  const themeColor = setting?.general.theme_color || DEFAULT_THEME_COLOR

  // 1. Apply Mode (Light/Dark)
  root.classList.remove('light', 'dark')

  if (theme === 'system' || !theme) {
    const systemTheme = systemThemeMedia.matches ? 'dark' : 'light'
    root.classList.add(systemTheme)
  } else {
    root.classList.add(theme)
  }

  // 2. Apply Theme Color
  root.setAttribute('data-theme', themeColor)

  // 3. 同步到 localStorage（供 splashscreen 使用）
  if (theme !== 'system') {
    localStorage.setItem('uc-theme', theme)
  } else {
    localStorage.removeItem('uc-theme')
  }
  localStorage.setItem('uc-theme-color', themeColor)
}
```

**Step 2: 验证代码正确性**

Run: `grep "localStorage.setItem('uc-theme" src/contexts/SettingContext.tsx`
Expected: 输出包含新添加的 localStorage 同步代码

**Step 3: Commit**

```bash
git add src/contexts/SettingContext.tsx
git commit -m "feat: sync theme to localStorage for splashscreen"
```

---

### Task 8: 移除 LoadingScreen 引用

**Files:**

- Modify: `src/App.tsx`

**Step 1: 移除 LoadingScreen 导入**

编辑 `src/App.tsx`，找到第 7 行：

删除：

```typescript
import { LoadingScreen } from '@/components/LoadingScreen'
```

**Step 2: 修改渲染逻辑**

编辑 `src/App.tsx`，找到约 208-213 行，将：

```typescript
  // Show loading screen if status not loaded yet (no TitleBar during loading)
  if (!statusLoaded) {
    return (
      <LoadingScreen className={fadingOut ? 'opacity-0 transition-opacity duration-300' : ''} />
    )
  }
```

改为：

```typescript
// Show loading screen if status not loaded yet (no TitleBar during loading)
if (!statusLoaded) {
  return null // splashscreen.html shown by Tauri before app loads
}
```

**Step 3: 验证无 LoadingScreen 引用**

Run: `grep "LoadingScreen" src/App.tsx`
Expected: 只在注释中找到 `LoadingScreen`，无实际引用

**Step 4: Commit**

```bash
git add src/App.tsx
git commit -m "refactor: remove LoadingScreen, use splashscreen.html instead"
```

---

### Task 9: 清理 LoadingScreen 组件和导出

**Files:**

- Delete: `src/components/LoadingScreen.tsx`
- Modify: `src/components/index.ts`

**Step 1: 删除 LoadingScreen 组件文件**

Run: `rm src/components/LoadingScreen.tsx`

**Step 2: 移除组件导出**

编辑 `src/components/index.ts`，删除第 19 行：

删除：

```typescript
export { LoadingScreen } from './LoadingScreen'
```

**Step 3: 验证无 LoadingScreen 引用**

Run: `grep -r "LoadingScreen" src/`
Expected: 无任何引用（除了可能的注释）

**Step 4: Commit**

```bash
git add src/components/index.ts
git commit -m "refactor: remove LoadingScreen component and export"
```

---

### Task 10: 清理 globals.css 中的动画定义

**Files:**

- Modify: `src/styles/globals.css`

**Step 1: 移除 LoadingScreen 动画注释和定义**

编辑 `src/styles/globals.css`，删除约 250-281 行：

删除：

```css
/* LoadingScreen animations */
@keyframes fade-in-up {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-fade-in-up {
  animation: fade-in-up 0.6s cubic-bezier(0.4, 0, 0.2, 1) both;
}

@keyframes pulse-wave {
  0%,
  100% {
    opacity: 0.4;
    transform: scale(0.8);
  }
  50% {
    opacity: 1;
    transform: scale(1.2);
  }
}

.animate-pulse-wave {
  animation: pulse-wave 1.2s ease-in-out infinite;
}
```

**Step 2: 验证文件正确性**

Run: `bun run build`
Expected: 构建成功，无错误

**Step 3: Commit**

```bash
git add src/styles/globals.css
git commit -m "refactor: remove unused LoadingScreen animations from globals.css"
```

---

## 测试与验证

### Task 11: 测试所有主题显示

**Files:**

- Test: 手动测试

**Step 1: 测试 zinc 主题**

1. 运行 `bun run dev`
2. 在浏览器中手动切换到 zinc + light 模式
3. 重启应用
4. 验证 splashscreen 使用 zinc light 主题

**Step 2: 测试 catppuccin 主题**

1. 切换到 catppuccin + dark 模式
2. 重启应用
3. 验证 splashscreen 使用 catppuccin dark 主题

**Step 3: 测试 claude 和 t3chat 主题**

重复上述步骤验证 claude 和 t3chat 主题

**Step 4: 测试系统默认模式**

1. 设置主题为 "system"
2. 验证 splashscreen 跟随系统主题

**Step 5: 验证动画流畅**

确认字母淡入和点脉冲动画流畅运行

**Step 6: 测试系统主题变化**

1. 设置主题为 "system"
2. 在运行时更改系统主题
3. 验证 splashscreen 实时响应

**Step 7: 检查文件大小**

Run: `ls -lh public/splashscreen.html`
Expected: 文件小于 50 KB

**Step 8: 记录测试结果**

创建测试检查清单：

- [ ] zinc 主题正确显示（light 模式）
- [ ] zinc 主题正确显示（dark 模式）
- [ ] catppuccin 主题正确显示（light 模式）
- [ ] catppuccin 主题正确显示（dark 模式）
- [ ] claude 主题正确显示（light 模式）
- [ ] claude 主题正确显示（dark 模式）
- [ ] t3chat 主题正确显示（light 模式）
- [ ] t3chat 主题正确显示（dark 模式）
- [ ] system 模式跟随系统主题
- [ ] 动画流畅（字母淡入、点脉冲）
- [ ] 系统主题变化时实时响应
- [ ] Tauri API 失败时正确降级到 localStorage
- [ ] 文件大小 < 50 KB

---

### Task 12: 最终验证和清理

**Files:**

- Test: 手动测试

**Step 1: 运行完整构建**

Run: `bun run build`
Expected: 构建成功，生成 `dist/` 目录

**Step 2: 验证 splashscreen 在 dist 中**

Run: `ls -lh dist/splashscreen.html`
Expected: 文件存在

**Step 3: 运行 Tauri 开发模式测试**

Run: `bun tauri dev`
Expected: 应用启动，显示正确的主题 splashscreen

**Step 4: 代码审查**

Run: `git diff`
Expected: 查看所有变更，确认无意外修改

**Step 5: 最终提交**

```bash
git add -A
git commit -m "feat: complete splashscreen generator implementation

- Build script generates self-contained splashscreen.html
- Supports all 4 themes (zinc, catppuccin, claude, t3chat)
- Automatic theme detection (Tauri API > localStorage > system)
- Removed LoadingScreen React component
- Added localStorage sync for theme persistence"
```

---

## 后续优化（可选）

这些任务不在核心功能范围内，可在后续迭代中实现：

### Optional Task 1: 添加 CSS 压缩

**Files:**

- Modify: `scripts/generate-splash.ts`
- Modify: `package.json`

**描述:** 添加 `csso` 或 `cssnano` 在生成时压缩 CSS，减小文件大小

---

### Optional Task 2: 支持自定义品牌名称

**Files:**

- Modify: `src/splashscreen/template.html`
- Modify: `scripts/generate-splash.ts`

**描述:** 通过环境变量或配置文件支持自定义品牌名称和字母

---

### Optional Task 3: 添加加载进度条

**Files:**

- Modify: `src/splashscreen/template.html`
- Modify: `src-tauri/src/main.rs`

**描述:** 从主应用接收加载进度事件，显示进度条

---

## 实施检查清单

在开始实施前，确保：

- [ ] 已阅读设计文档 (`docs/plans/2026-01-18-splashscreen-generator-design.md`)
- [ ] 理解主题系统的 CSS 变量结构
- [ ] 理解 Tauri API 的异步调用模式
- [ ] 确认 glob 依赖已安装

完成实施后，验证：

- [ ] 所有 4 个主题的 8 种变体正确显示
- [ ] 动画流畅运行
- [ ] 系统主题变化时实时响应
- [ ] Tauri API 失败时正确降级
- [ ] 文件大小 < 50 KB
- [ ] 无 LoadingScreen 引用残留
- [ ] 构建流程正确集成 predev/prebuild 钩子
