import { Inbox } from 'lucide-react'
import React, { useMemo, useState, useEffect, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import ClipboardItem from './ClipboardItem'
import ClipboardSelectionActionBar from './ClipboardSelectionActionBar'
import DeleteConfirmDialog from './DeleteConfirmDialog'
import {
  getDisplayType,
  ClipboardItemResponse,
  Filter,
  ClipboardTextItem,
  ClipboardImageItem,
  ClipboardLinkItem,
  ClipboardCodeItem,
  ClipboardFileItem,
} from '@/api/clipboardItems'
import { Skeleton } from '@/components/ui/skeleton'
import { toast } from '@/components/ui/sonner'
import { useShortcut } from '@/hooks/useShortcut'
import { useAppDispatch, useAppSelector } from '@/store/hooks'
import {
  removeClipboardItem,
  copyToClipboard,
  toggleFavoriteItem,
} from '@/store/slices/clipboardSlice'

interface DisplayClipboardItem {
  id: string
  type: 'text' | 'image' | 'link' | 'code' | 'file' | 'unknown'
  time: string
  isDownloaded?: boolean
  isFavorited?: boolean
  content:
    | ClipboardTextItem
    | ClipboardImageItem
    | ClipboardLinkItem
    | ClipboardCodeItem
    | ClipboardFileItem
    | null
  device?: string
}

interface ClipboardContentProps {
  filter: Filter
  searchQuery?: string
}

const ClipboardContent: React.FC<ClipboardContentProps> = ({ filter, searchQuery = '' }) => {
  const { t } = useTranslation()

  // Use Redux state and dispatch
  const dispatch = useAppDispatch()
  const { items: reduxItems, loading } = useAppSelector(state => state.clipboard)

  // Local state for converted display items
  const [clipboardItems, setClipboardItems] = useState<DisplayClipboardItem[]>([])

  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())
  const [lastSelectedIndex, setLastSelectedIndex] = useState<number | null>(null)
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [copySuccess, setCopySuccess] = useState(false)

  // 注册 ESC 取消选择快捷键
  useShortcut({
    key: 'esc',
    scope: 'clipboard',
    enabled: selectedIds.size > 0, // 只在有选中时启用
    handler: () => {
      setSelectedIds(new Set())
      setLastSelectedIndex(null)
    },
  })

  // 选中状态下的快捷键：复制/收藏/删除
  useShortcut({
    key: 'c',
    scope: 'clipboard',
    enabled: selectedIds.size > 0,
    handler: () => {
      void handleBatchCopy()
    },
    preventDefault: false,
  })

  useShortcut({
    key: 's',
    scope: 'clipboard',
    enabled: selectedIds.size > 0,
    handler: () => {
      void handleBatchToggleFavorite()
    },
    preventDefault: false,
  })

  useShortcut({
    key: 'd',
    scope: 'clipboard',
    enabled: selectedIds.size > 0,
    handler: () => {
      void handleBatchDelete()
    },
    preventDefault: false,
  })

  // Convert clipboard item to display item
  const convertToDisplayItem = useCallback(
    (item: ClipboardItemResponse): DisplayClipboardItem => {
      console.log(t('clipboard.content.logs.convertingItem'), item)
      // Get type suitable for UI display
      const type = getDisplayType(item.item)

      // Format time (active_time is already in milliseconds from backend)
      const activeTime = new Date(item.active_time)
      const now = new Date()
      const diffMs = now.getTime() - activeTime.getTime()
      const diffMins = Math.round(diffMs / 60000)

      let timeString: string
      if (diffMins < 1) {
        timeString = t('clipboard.time.justNow')
      } else if (diffMins < 60) {
        timeString = t('clipboard.time.minutesAgo', { minutes: diffMins })
      } else if (diffMins < 1440) {
        timeString = t('clipboard.time.hoursAgo', { hours: Math.floor(diffMins / 60) })
      } else {
        timeString = t('clipboard.time.daysAgo', { days: Math.floor(diffMins / 1440) })
      }

      // Create display item
      return {
        id: item.id,
        type,
        time: timeString,
        isDownloaded: item.is_downloaded,
        isFavorited: item.is_favorited,
        content:
          type === 'text'
            ? item.item.text
            : type === 'image'
              ? item.item.image
              : type === 'link'
                ? item.item.link
                : type === 'code'
                  ? item.item.code
                  : type === 'file'
                    ? item.item.file
                    : null,
        device: item.device_id,
      }
    },
    [t]
  )

  // 监听 Redux 中的 items 变化,转换为显示项目
  useEffect(() => {
    console.log(t('clipboard.content.filterCondition'), filter)
    console.log(t('clipboard.content.queryResults'), reduxItems)

    if (reduxItems && reduxItems.length > 0) {
      let items: DisplayClipboardItem[] = reduxItems.map(convertToDisplayItem)

      // Apply filter
      if (filter === Filter.Favorited) {
        items = items.filter(it => it.isFavorited)
      }

      // Apply search query
      if (searchQuery.trim()) {
        const query = searchQuery.toLowerCase().trim()
        items = items.filter(it => {
          // Search in text content
          if (it.type === 'text' && it.content) {
            const textItem = it.content as ClipboardTextItem
            return textItem.display_text?.toLowerCase().includes(query)
          }
          // Search in code content
          if (it.type === 'code' && it.content) {
            const codeItem = it.content as ClipboardCodeItem
            return codeItem.code?.toLowerCase().includes(query)
          }
          // Search in link URL
          if (it.type === 'link' && it.content) {
            const linkItem = it.content as ClipboardLinkItem
            return linkItem.url?.toLowerCase().includes(query)
          }
          // Search in file names
          if (it.type === 'file' && it.content) {
            const fileItem = it.content as ClipboardFileItem
            return fileItem.file_names?.some(name => name.toLowerCase().includes(query))
          }
          return false
        })
      }

      setClipboardItems(items)
      console.log(t('clipboard.content.convertedItems'), items)
    } else {
      setClipboardItems([])
      console.log(t('clipboard.content.noItemsFound'))
    }
  }, [reduxItems, filter, searchQuery, t, convertToDisplayItem])

  // 处理复制到剪贴板
  const handleCopyItem = async (itemId: string) => {
    try {
      console.log(`${t('clipboard.content.logs.copyItem')} ${itemId}`)
      const result = await dispatch(copyToClipboard(itemId)).unwrap()
      if (result.success) {
        setCopySuccess(true)
        setTimeout(() => setCopySuccess(false), 1500)
      }
      return result.success
    } catch (err) {
      console.error(t('clipboard.content.logs.copyToClipboardFailed'), err)

      // 显示复制失败的 toast 提示
      toast.error(t('clipboard.errors.copyFailed'), {
        description: err instanceof Error ? err.message : t('clipboard.errors.unknown'),
      })

      return false
    }
  }

  // 当列表变化时，清理已经不存在的选择
  useEffect(() => {
    setSelectedIds(prev => {
      if (prev.size === 0) return prev
      const valid = new Set(clipboardItems.map(i => i.id))
      const next = new Set<string>()
      for (const id of prev) {
        if (valid.has(id)) next.add(id)
      }
      return next
    })
    if (lastSelectedIndex !== null && lastSelectedIndex >= clipboardItems.length) {
      setLastSelectedIndex(null)
    }
  }, [clipboardItems, lastSelectedIndex])

  const handleSelect = (id: string, index: number, event: React.MouseEvent<HTMLDivElement>) => {
    setSelectedIds(prev => {
      const next = new Set(prev)

      const isMultiToggle = event.metaKey || event.ctrlKey
      const isRange = event.shiftKey

      if (isRange && lastSelectedIndex !== null) {
        const start = Math.min(lastSelectedIndex, index)
        const end = Math.max(lastSelectedIndex, index)
        const rangeIds = clipboardItems.slice(start, end + 1).map(i => i.id)

        if (!isMultiToggle) next.clear()
        for (const rid of rangeIds) next.add(rid)
        return next
      }

      if (isMultiToggle) {
        if (next.has(id)) next.delete(id)
        else next.add(id)
        return next
      }

      // 默认：单选
      if (next.size === 1 && next.has(id)) return next
      next.clear()
      next.add(id)
      return next
    })

    setLastSelectedIndex(index)
  }

  const selectedItems = useMemo(() => {
    if (selectedIds.size === 0) return []
    return clipboardItems.filter(it => selectedIds.has(it.id))
  }, [clipboardItems, selectedIds])

  const favoriteIntent: 'favorite' | 'unfavorite' = useMemo(() => {
    if (selectedItems.length === 0) return 'favorite'
    const allFavorited = selectedItems.every(it => Boolean(it.isFavorited))
    return allFavorited ? 'unfavorite' : 'favorite'
  }, [selectedItems])

  const handleBatchCopy = async (): Promise<boolean> => {
    if (selectedItems.length === 0) return false
    if (selectedItems.length === 1) {
      return handleCopyItem(selectedItems[0].id)
    }

    const toText = (it: DisplayClipboardItem): string => {
      switch (it.type) {
        case 'text':
          return (it.content as ClipboardTextItem)?.display_text ?? ''
        case 'code':
          return (it.content as ClipboardCodeItem)?.code ?? ''
        case 'link':
          return (it.content as ClipboardLinkItem)?.url ?? ''
        case 'file': {
          const names = (it.content as ClipboardFileItem)?.file_names ?? []
          return names.length > 0 ? names.join('\n') : '[文件]'
        }
        case 'image':
          return '[图片]'
        default:
          return ''
      }
    }

    const text = selectedItems.map(toText).filter(Boolean).join('\n\n')
    if (!text) return false
    try {
      await navigator.clipboard.writeText(text)
      // 批量复制成功后高亮第一个选中项并显示复制成功反馈
      setCopySuccess(true)
      setTimeout(() => setCopySuccess(false), 1500)
      return true
    } catch (e) {
      console.error('批量复制失败:', e)
      return false
    }
  }

  const handleBatchToggleFavorite = async () => {
    if (selectedItems.length === 0) return
    const targetFavorited = favoriteIntent === 'favorite'
    await Promise.all(
      selectedItems.map(it =>
        dispatch(toggleFavoriteItem({ id: it.id, isFavorited: targetFavorited }))
          .unwrap()
          .catch(e => console.error('设置收藏状态失败:', e))
      )
    )
  }

  const handleBatchDelete = async () => {
    if (selectedItems.length === 0) return
    setDeleteDialogOpen(true)
  }

  const handleConfirmDelete = async () => {
    for (const it of selectedItems) {
      try {
        await dispatch(removeClipboardItem(it.id)).unwrap()
      } catch (e) {
        console.error('删除失败:', e)
      }
    }
    setSelectedIds(new Set())
    setLastSelectedIndex(null)
  }

  // Skeleton loading state
  if (loading && clipboardItems.length === 0) {
    return (
      <div className="h-full overflow-y-auto scrollbar-thin px-4 pb-32 pt-2">
        <div className="flex flex-col gap-1">
          {Array.from({ length: 12 }).map((_, i) => (
            <Skeleton key={i} className="h-12 w-full rounded-lg" />
          ))}
        </div>
      </div>
    )
  }

  return (
    <div className="h-full relative">
      <div className="h-full overflow-y-auto scrollbar-thin px-4 pb-32 pt-2">
        {clipboardItems.length > 0 ? (
          <div className="flex flex-col">
            {clipboardItems.map((item, index) => (
              <ClipboardItem
                key={item.id}
                index={index + 1}
                type={item.type}
                time={item.time}
                device={item.device}
                content={item.content}
                isSelected={selectedIds.has(item.id)}
                onSelect={e => handleSelect(item.id, index, e)}
              />
            ))}
          </div>
        ) : (
          <div className="h-full flex flex-col items-center justify-center gap-6">
            <div className="flex flex-col items-center justify-center w-16 h-16 rounded-lg bg-muted/50 border border-dashed border-muted">
              <Inbox className="h-8 w-8 text-muted-foreground/60" />
            </div>
            <div className="text-center">
              <h3 className="text-base font-semibold text-foreground mb-1">
                {t('clipboard.content.noClipboardItems')}
              </h3>
              <p className="text-sm text-muted-foreground">
                {t('clipboard.content.emptyDescription')}
              </p>
            </div>
          </div>
        )}
      </div>

      <ClipboardSelectionActionBar
        selectedCount={selectedIds.size}
        favoriteIntent={favoriteIntent}
        copySuccess={copySuccess}
        onCopy={handleBatchCopy}
        onToggleFavorite={handleBatchToggleFavorite}
        onDelete={handleBatchDelete}
        onClearSelection={() => {
          setSelectedIds(new Set())
          setLastSelectedIndex(null)
        }}
      />

      <DeleteConfirmDialog
        open={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        onConfirm={handleConfirmDelete}
        count={selectedItems.length}
      />
    </div>
  )
}

export default ClipboardContent
