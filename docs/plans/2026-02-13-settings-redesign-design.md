# 设置页面重新设计方案

## 背景

当前 `SettingsPage.tsx` 采用侧边栏 + 内容区布局，共 7 个分类。存在以下问题：

- 标题 + 分割线模式在每个区块中重复手动实现
- 存在未使用的玻璃拟态组件（`SettingHeader.tsx`、`SettingFooter.tsx`）
- 侧边栏宽度不一致（20rem vs 10.625rem 最小值）
- `renderActiveSection` 使用 switch 语句，添加新分类需要多处修改

## 设计目标

- **体验优化**：更好的导航、更清晰的组织
- **解决痛点**：消除重复代码、统一风格、简化状态管理
- **保持简洁**：不需要搜索、不需要键盘快捷键、保持即时保存 + 静默

## 设计决策

| 决策项   | 选择                 |
| -------- | -------------------- |
| 导航结构 | 保持左侧边栏         |
| 搜索功能 | 不需要               |
| 视觉风格 | 延续 Shadcn 扁平设计 |
| 键盘导航 | 只要 ESC 返回        |
| 保存反馈 | 即时保存 + 静默      |

## 实现方案

### 1. 组件重构

#### SettingSectionHeader 组件

```tsx
// src/components/setting/SettingSectionHeader.tsx
interface Props {
  title: string
  description?: string
}

export const SettingSectionHeader: React.FC<Props> = ({ title, description }) => (
  <div className="flex items-center gap-2 mb-4">
    <span className="text-sm text-muted-foreground">{title}</span>
    {description && <span className="text-xs text-muted-foreground/60">{description}</span>}
    <div className="h-px flex-1 bg-border/50" />
  </div>
)
```

#### SettingCard 组件

```tsx
// src/components/setting/SettingCard.tsx
interface Props {
  title: string
  description?: string
  children: React.ReactNode
}

export const SettingCard: React.FC<Props> = ({ title, description, children }) => (
  <Card>
    <CardContent className="pt-6">
      <SettingSectionHeader title={title} description={description} />
      {children}
    </CardContent>
  </Card>
)
```

### 2. 配置驱动重构

```tsx
// src/pages/SettingsPage.tsx
import GeneralSection from '@/components/setting/GeneralSection'
import AppearanceSection from '@/components/setting/AppearanceSection'
// ... 其他导入

const SETTINGS_SECTIONS: Record<string, React.FC> = {
  general: GeneralSection,
  appearance: AppearanceSection,
  sync: SyncSection,
  security: SecuritySection,
  network: NetworkSection,
  storage: StorageSection,
  about: AboutSection,
}

const SETTINGS_CATEGORIES = [
  { id: 'general', label: '通用', icon: Settings },
  { id: 'appearance', label: '外观', icon: Palette },
  { id: 'sync', label: '同步', icon: RefreshCw },
  { id: 'security', label: '安全', icon: Shield },
  { id: 'network', label: '网络', icon: Wifi },
  { id: 'storage', label: '存储', icon: HardDrive },
  { id: 'about', label: '关于', icon: Info },
]

// 渲染
const ActiveSection = SETTINGS_SECTIONS[activeCategory]
return (
  <SettingContentLayout>
    <ActiveSection />
  </SettingContentLayout>
)
```

### 3. 布局与样式优化

**侧边栏宽度统一**

- `SettingsPage.tsx`: `--sidebar-width: 16rem`
- `SettingsSidebar.tsx`: 移除 `min-w-[10.625rem]`

**内容区**

- 内边距：`p-8` → `p-6`
- 保持 `ScrollArea` 和 `space-y-6` 间距

### 4. 状态管理

保持现有双层状态模式（本地 state + useSetting），这是即时 UI 反馈 + 后端同步的合理方案。

## 文件变更

### 新增

| 文件                                              | 用途               |
| ------------------------------------------------- | ------------------ |
| `src/components/setting/SettingSectionHeader.tsx` | 统一的区块头部组件 |
| `src/components/setting/SettingCard.tsx`          | 统一的区块容器组件 |

### 修改

| 文件                                           | 改动                            |
| ---------------------------------------------- | ------------------------------- |
| `src/pages/SettingsPage.tsx`                   | 配置驱动重构、统一 sidebar 宽度 |
| `src/components/setting/SettingsSidebar.tsx`   | 从配置生成分类列表              |
| `src/components/setting/GeneralSection.tsx`    | 使用 SettingCard                |
| `src/components/setting/AppearanceSection.tsx` | 使用 SettingCard                |
| `src/components/setting/SyncSection.tsx`       | 使用 SettingCard                |
| `src/components/setting/SecuritySection.tsx`   | 使用 SettingCard                |
| `src/components/setting/NetworkSection.tsx`    | 使用 SettingCard                |
| `src/components/setting/StorageSection.tsx`    | 使用 SettingCard                |
| `src/components/setting/AboutSection.tsx`      | 使用 SettingCard                |

### 删除

| 文件                                       | 原因   |
| ------------------------------------------ | ------ |
| `src/components/setting/SettingHeader.tsx` | 未使用 |
| `src/components/setting/SettingFooter.tsx` | 未使用 |

## 实施顺序

1. 创建 `SettingSectionHeader` 和 `SettingCard` 组件
2. 重构 `SettingsPage.tsx` 为配置驱动
3. 更新 `SettingsSidebar.tsx` 从配置生成列表
4. 逐个更新 Section 组件使用新组件
5. 删除未使用的组件
6. 测试所有设置页面功能
