import { createContext, useCallback, useContext, useMemo, useRef, useState, ReactNode } from 'react'
import { ShortcutScope } from '@/shortcuts/definitions'
import { ShortcutLayer, LAYER_ORDER } from '@/shortcuts/layers'

/**
 * 快捷键上下文接口
 */
interface ShortcutContextType {
  /** 当前激活的作用域（由 layer + priority 决定） */
  activeScope: ShortcutScope
  /** 当前激活的 layer（由 layer + priority 决定） */
  activeLayer: ShortcutLayer
  /** 当前激活的 priority（由 layer + priority 决定） */
  activePriority: number
  /** 推入一个 layer 上下文，返回 token 用于释放 */
  pushLayer: (entry: { layer: ShortcutLayer; scope: ShortcutScope; priority?: number }) => string
  /** 释放一个 layer 上下文（可乱序释放） */
  popLayer: (token: string) => void
}

/**
 * 快捷键上下文
 */
const ShortcutContext = createContext<ShortcutContextType | undefined>(undefined)

type LayerEntry = {
  token: string
  layer: ShortcutLayer
  scope: ShortcutScope
  priority: number
  order: number
  isBase?: boolean
}

/**
 * 快捷键 Provider 组件属性
 */
interface ShortcutProviderProps {
  children: ReactNode
}

/**
 * 快捷键 Provider 组件
 * 管理 layer（modal/page/global）+ priority 的上下文栈，并据此计算当前激活的作用域
 */
export const ShortcutProvider: React.FC<ShortcutProviderProps> = ({ children }) => {
  const baseToken = '__shortcut_base__'
  const orderRef = useRef(1)

  const [entries, setEntries] = useState<LayerEntry[]>([
    {
      token: baseToken,
      layer: 'global',
      scope: 'global',
      priority: 0,
      order: 0,
      isBase: true,
    },
  ])

  const pushLayer = useCallback(
    ({
      layer,
      scope,
      priority = 0,
    }: {
      layer: ShortcutLayer
      scope: ShortcutScope
      priority?: number
    }): string => {
      const token =
        typeof crypto !== 'undefined' && 'randomUUID' in crypto
          ? crypto.randomUUID()
          : `${Date.now()}_${Math.random().toString(16).slice(2)}`
      const order = orderRef.current++

      setEntries(prev => [...prev, { token, layer, scope, priority, order }])

      return token
    },
    []
  )

  const popLayer = useCallback((token: string): void => {
    if (token === baseToken) return
    setEntries(prev => prev.filter(e => e.token !== token))
  }, [])

  const activeEntry = useMemo<LayerEntry>(() => {
    const [first] = entries
    if (!first) {
      return {
        token: baseToken,
        layer: 'global',
        scope: 'global',
        priority: 0,
        order: 0,
        isBase: true,
      }
    }

    return entries.reduce((best, current) => {
      const bestLayerOrder = LAYER_ORDER[best.layer]
      const currentLayerOrder = LAYER_ORDER[current.layer]

      if (currentLayerOrder !== bestLayerOrder) {
        return currentLayerOrder > bestLayerOrder ? current : best
      }
      if (current.priority !== best.priority) {
        return current.priority > best.priority ? current : best
      }
      return current.order > best.order ? current : best
    }, first)
  }, [entries])

  return (
    <ShortcutContext.Provider
      value={{
        activeScope: activeEntry.scope,
        activeLayer: activeEntry.layer,
        activePriority: activeEntry.priority,
        pushLayer,
        popLayer,
      }}
    >
      {children}
    </ShortcutContext.Provider>
  )
}

/**
 * 使用快捷键上下文的 Hook
 */
export const useShortcutContext = (): ShortcutContextType => {
  const context = useContext(ShortcutContext)
  if (context === undefined) {
    throw new Error('useShortcutContext must be used within ShortcutProvider')
  }
  return context
}
