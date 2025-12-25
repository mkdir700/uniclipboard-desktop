import { useHotkeys } from "react-hotkeys-hook";
import { useShortcutContext } from "@/contexts/ShortcutContext";
import { ShortcutScope } from "@/shortcuts/definitions";

/**
 * useShortcut Hook 选项
 */
interface UseShortcutOptions {
  /** 快捷键组合，如 "esc", "cmd+a" */
  key: string;
  /** 作用域 */
  scope: ShortcutScope;
  /** 是否启用（可选，默认 true） */
  enabled?: boolean;
  /** 触发时的处理函数 */
  handler: () => void;
  /** 是否阻止默认行为（可选，默认 true） */
  preventDefault?: boolean;
}

/**
 * 快捷键注册 Hook
 *
 * 基于 react-hotkeys-hook 封装，支持作用域隔离和条件启用
 *
 * @example
 * ```tsx
 * useShortcut({
 *   key: "esc",
 *   scope: "clipboard",
 *   enabled: selectedIds.size > 0,
 *   handler: () => setSelectedIds(new Set()),
 * });
 * ```
 */
export const useShortcut = ({
  key,
  scope,
  enabled = true,
  handler,
  preventDefault = true,
}: UseShortcutOptions): void => {
  const { activeScope } = useShortcutContext();

  // 只有当作用域匹配且启用时才注册快捷键
  const isActive = activeScope === scope && enabled;

  useHotkeys(
    key,
    handler,
    {
      enabled: isActive,
      preventDefault,
      enableOnFormTags: false,
      enableOnContentEditable: false,
    },
    [key, scope, enabled, activeScope, handler, preventDefault]
  );
};
