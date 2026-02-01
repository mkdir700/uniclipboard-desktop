# Devices Page Redesign Plan

## TL;DR

> **Quick Summary**: 把 Devices 页面重构为“设备列表 + 空态插画 + 末尾添加入口”的结构，并把配对请求改为全局 toast，点击后才弹出配对请求 modal。
>
> **Deliverables**:
>
> - 移除 `DevicesPage` 顶部 header 与“无请求/无设备”虚线区域
> - 新的空态（Lucide 图标 + 文案 + 添加按钮）与“列表末尾添加设备”入口
> - 全局配对请求 toast（可点击打开 modal，手动关闭，`duration: Infinity`）
>
> **Estimated Effort**: Medium
> **Parallel Execution**: NO - sequential
> **Critical Path**: Task 1 → Task 2 → Task 3

---

## Context

### Original Request

重设计 `src/pages/DevicesPage.tsx`：移除 header、配对请求改为 toast、删除虚线空态框、列表末尾添加入口、无设备时居中空态插画与添加按钮；toast 点击弹 modal，modal 全局展示。

### Interview Summary

**Key Discussions**:

- 配对请求 toast 常驻（手动关闭），不自动弹出 modal。
- 空态插画使用 Lucide 图标风格。
- 不新增自动化测试，保留手动验证偏好。

**Research Findings**:

- Sonner Toaster 挂在 `src/App.tsx`，`toast` 从 `src/components/ui/toast.ts` 统一导出。`duration` 已有使用示例。 (`src/App.tsx:65-66`, `src/components/ui/sonner.tsx:6-44`, `src/components/ui/toast.ts:1-3`, `src/pages/DashboardPage.tsx:182-186`)
- P2P 全局 Provider 已存在，`GlobalPairingRequestDialog` 与 `PairingPinDialog` 在 `App` 全局挂载。 (`src/contexts/P2PContext.tsx:15-138`, `src/App.tsx:28-52`)
- Devices 页内部有重复的配对监听与本地状态。 (`src/pages/DevicesPage.tsx:30-94`)
- 设备空态目前是虚线框，剪贴板空态有“图标 + 标题 + 描述”样式可复用。 (`src/components/device/OtherDevice.tsx:134-143`, `src/components/clipboard/ClipboardContent.tsx:401-433`)

### Metis Review

**Identified Gaps** (addressed):

- 多请求与文案策略未明确 → 默认复用现有 i18n key，并保持单请求覆盖模式。
- toast 点击区与关闭按钮的冲突 → 明确关闭按钮阻止冒泡。
- “不自动打开 modal”容易误实现 → 计划强制仅在 toast 点击时打开。

### Defaults Applied

- i18n 文案复用 `pairing.globalRequest.*` 与 `pairing.requests.*`（避免新增 key）。
- 多请求采用“最新覆盖”模式（不做队列/聚合）。
- 空态 Lucide 图标默认使用 `Smartphone` 风格。
- toast 手动关闭默认等同于“忽略/拒绝”，并清理 pending request。

---

## Work Objectives

### Core Objective

将 Devices 页的 UI 与配对请求入口重构为“toast 通知 + 点击触发 modal + 中央空态 + 列表末尾添加入口”，确保全局可用、交互一致。

### Concrete Deliverables

- 移除 `DeviceHeader` 使用与配对请求“无请求”区域。
- 设备列表容器重新组织：有设备就列出，没设备就居中空态。
- 在列表末尾新增“添加设备”入口；无设备时入口在空态中。
- 全局配对请求 toast：可点击打开 `GlobalPairingRequestDialog`，`duration: Infinity`，支持手动关闭。

### Definition of Done

- `DevicesPage` 不再渲染 `DeviceHeader` 与配对请求虚线框。
- `OtherDevice` 不再显示虚线空态框，改为新空态样式。
- 配对请求到来时会展示 toast，点击 toast 后出现 modal；不自动弹窗。
- Toast 可手动关闭（并清理/拒绝对应请求）。

### Must Have

- Toast 可点击打开 modal（后续流程与现有一致）。
- Toast 常驻，`duration: Infinity`。
- 空态居中展示且包含“添加设备”按钮。

### Must NOT Have (Guardrails)

- 不改动 P2P 后端或 API 行为，仅调整前端状态与展示。
- 不引入新的 toast 库；继续使用 Sonner 现有封装。
- 不使用固定像素布局；仅用 Tailwind 实用类或 `rem`。
- 不自动弹出配对请求 modal（仅由 toast 点击触发）。

---

## Verification Strategy (MANDATORY)

### Test Decision

- **Infrastructure exists**: 未针对该 UI 提供测试
- **User wants tests**: NO
- **Framework**: None

> 由于用户要求手动验证，但本计划必须提供可执行的自动化验收，采用 **静态可执行检查** 作为最低自动验证标准（不新增测试）。

---

## Execution Strategy

### Parallel Execution Waves

Wave 1 (Start Immediately):
├── Task 1: 全局配对请求 toast + P2PContext 调整

Wave 2 (After Wave 1):
└── Task 2: DevicesPage 结构重组（移除 header、请求区、移除本地监听）

Wave 3 (After Wave 2):
└── Task 3: 列表末尾添加入口 + 空态样式改造

Critical Path: Task 1 → Task 2 → Task 3

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
| ---- | ---------- | ------ | -------------------- |
| 1    | None       | 2      | None                 |
| 2    | 1          | 3      | None                 |
| 3    | 2          | None   | None                 |

---

## TODOs

> 每个任务包含实现 + 验收（可执行）。

- [x] 1. 全局配对请求 toast 与 P2PContext 行为改造

  **What to do**:
  - 在 `P2PContext` 中仅记录 `pendingRequest`，不自动打开 modal（移除 `setShowRequestDialog(true)` 的即时弹窗）。
  - 新增 `openRequestDialog()` 或等价动作，仅在 toast 点击时设置 `showRequestDialog = true`。
  - 在 `P2PProvider` 里触发 Sonner toast，`duration: Infinity`，支持手动关闭；点击 toast 调用 `openRequestDialog()`。
  - 手动关闭：默认视为“忽略/拒绝”并清理 `pendingRequest`（必要时调用 `rejectRequest`，并阻止点击冒泡影响 toast 点击）。

  **Must NOT do**:
  - 不新增新的 toast 系统或重写 Toaster。
  - 不修改后端接口或事件名称。

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: 全局状态与事件流调整，需谨慎影响范围。
  - **Skills**: `systematic-debugging`, `executing-plans`
    - `systematic-debugging`: 需要避免重复触发与状态竞争
    - `executing-plans`: 多文件一致性修改
  - **Skills Evaluated but Omitted**:
    - `test-driven-development`: 用户不需要新增测试

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Task 2
  - **Blocked By**: None

  **References**:
  - `src/contexts/P2PContext.tsx:24-59` - 当前在事件回调中立即打开 request dialog 的逻辑
  - `src/contexts/P2PContext.tsx:71-95` - accept/reject 请求的现有实现
  - `src/types/p2p.ts:7-19` - P2PContext 类型定义（需要扩展新动作）
  - `src/components/ui/toast.ts:1-3` - Sonner toast 统一导出入口
  - `src/components/ui/sonner.tsx:6-44` - Toaster 配置与样式
  - `src/pages/DashboardPage.tsx:182-186` - `duration` 设定示例
  - `src/components/GlobalPairingRequestDialog.tsx:22-87` - modal 展示与 accept/reject 文案复用

  **Acceptance Criteria**:
  - [ ] `rg "duration:\s*Infinity" src` 命中配对 toast 设置
  - [ ] `rg "showRequestDialog" src/contexts/P2PContext.tsx` 不再在事件监听中直接置为 `true`
  - [ ] `rg "openRequestDialog|showRequestDialog" src/contexts/P2PContext.tsx` 可找到 toast 点击触发的打开逻辑
  - [ ] `rg "toast\(" src/contexts/P2PContext.tsx` 可找到配对请求 toast 的触发点

- [x] 2. DevicesPage 结构重组（移除 header、请求区、局部监听）

  **What to do**:
  - 移除 `DeviceHeader` 与“配对请求”区块（含“无请求”虚线框）。
  - 删除 DevicesPage 内部的配对监听与本地 `pendingP2PRequest` / `showPinDialog` 状态，避免重复监听。
  - 保留 `PairingDialog` 作为主动发起配对入口，但将触发入口移动到列表末尾/空态按钮。

  **Must NOT do**:
  - 不改变 `PairingDialog` 的现有配对流程逻辑。
  - 不改动 `PairingPinDialog` 的流程与文案。

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: 页面布局与结构重组
  - **Skills**: `frontend-ui-ux`
    - `frontend-ui-ux`: 保持 UI 视觉一致性、空态美化
  - **Skills Evaluated but Omitted**:
    - `vercel-react-best-practices`: 非 Next.js 性能优化场景

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Task 3
  - **Blocked By**: Task 1

  **References**:
  - `src/pages/DevicesPage.tsx:51-211` - 现有 header 与配对请求区块
  - `src/pages/DevicesPage.tsx:30-94` - 本地配对监听逻辑
  - `src/components/device/Header.tsx:10-39` - 当前 header 与添加按钮样式

  **Acceptance Criteria**:
  - [ ] `rg "DeviceHeader" src/pages/DevicesPage.tsx` 无匹配
  - [ ] `rg "devices.sections.noRequests" src/pages/DevicesPage.tsx` 无匹配
  - [ ] `rg "onP2PPairingRequest|onP2PPinReady" src/pages/DevicesPage.tsx` 无匹配

- [x] 3. 设备列表末尾添加入口 + 空态重构

  **What to do**:
  - 在设备列表末尾增加“添加设备”入口（按钮或卡片风格）。
  - 当没有已配对设备时，显示居中空态：Lucide 图标 + 标题 + 描述 + 添加按钮。
  - 空态视觉风格对齐 `ClipboardContent` 的空态结构（图标背景 + 文案层级）。
  - 不使用虚线空态框；列表容器保持滚动区域。

  **Must NOT do**:
  - 不引入新的插画资产。
  - 不使用固定像素高度，避免硬编码高度。

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: 空态视觉与列表布局优化
  - **Skills**: `frontend-ui-ux`
    - `frontend-ui-ux`: 空态插画与按钮排版
  - **Skills Evaluated but Omitted**:
    - `test-driven-development`: 用户不需要新增测试

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: None
  - **Blocked By**: Task 2

  **References**:
  - `src/components/device/OtherDevice.tsx:134-143` - 现有虚线空态
  - `src/components/device/DeviceList.tsx:5-13` - 设备列表组件结构
  - `src/components/clipboard/ClipboardContent.tsx:401-433` - 空态结构与视觉参考
  - `src/components/device/CurrentDevice.tsx:62-104` - 设备卡片视觉风格参考

  **Acceptance Criteria**:
  - [ ] `rg "暂无已配对的设备" src/components/device/OtherDevice.tsx` 无匹配
  - [ ] `rg "border-dashed" src/components/device/OtherDevice.tsx` 无匹配（空态虚线框移除）
  - [ ] `rg "devices.addNew" src/components/device src/pages` 可找到新增入口

---

## Commit Strategy

| After Task | Message                                               | Files                           | Verification                                  |
| ---------- | ----------------------------------------------------- | ------------------------------- | --------------------------------------------- |
| 1          | `feat(p2p): move pairing request to toast entrypoint` | P2PContext + toast hook         | `rg "duration:\s*Infinity" src`               |
| 2-3        | `feat(devices): redesign page layout and empty state` | DevicesPage + device components | `rg "DeviceHeader" src/pages/DevicesPage.tsx` |

---

## Success Criteria

### Verification Commands

```bash
rg "DeviceHeader" src/pages/DevicesPage.tsx
rg "devices.sections.noRequests" src/pages/DevicesPage.tsx
rg "onP2PPairingRequest|onP2PPinReady" src/pages/DevicesPage.tsx
rg "duration:\s*Infinity" src
rg "暂无已配对的设备" src/components/device/OtherDevice.tsx
```

### Final Checklist

- [ ] Devices 页无 header、无配对请求占位框
- [ ] 空态居中且含添加按钮
- [ ] Toast 仅点击后打开 modal，且可手动关闭
- [ ] P2P 业务逻辑未被改动
