import { Check, Copy, Star, Trash2, X } from 'lucide-react'
import React, { useMemo } from 'react'
import { Button } from '@/components/ui/button'
import { Kbd } from '@/components/ui/kbd'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'

const FAVORITES_UI_ENABLED = false

interface ClipboardSelectionActionBarProps {
  selectedCount: number
  favoriteIntent: 'favorite' | 'unfavorite'
  showHotkeys?: boolean
  copySuccess?: boolean
  onCopy: () => Promise<boolean> | boolean
  onToggleFavorite: () => Promise<void> | void
  onDelete: () => Promise<void> | void
  onClearSelection?: () => void
}

const ClipboardSelectionActionBar: React.FC<ClipboardSelectionActionBarProps> = ({
  selectedCount,
  favoriteIntent,
  showHotkeys = true,
  copySuccess = false,
  onCopy,
  onToggleFavorite,
  onDelete,
  onClearSelection,
}) => {
  const favoriteTitle = useMemo(() => {
    if (favoriteIntent === 'unfavorite') return '取消收藏'
    return '收藏'
  }, [favoriteIntent])

  if (selectedCount === 0) {
    return null
  }

  return (
    <TooltipProvider delayDuration={0}>
      <div className="absolute bottom-6 left-0 right-0 z-20 px-4 flex justify-center">
        <div className="glass-strong border border-border/60 rounded-full shadow-lg inline-flex items-center gap-1 px-2 py-1.5">
          {/* Selection count badge */}
          <div className="flex items-center justify-center w-8 h-8 rounded-full bg-primary/10 text-primary text-sm font-semibold mr-1">
            {selectedCount}
          </div>

          <div className="w-px h-5 bg-border/50 mx-1" />

          {/* Action buttons */}
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size="sm"
                variant="ghost"
                className="h-8 w-8 p-0 rounded-full hover:bg-primary/10 hover:text-primary"
                onClick={onCopy}
              >
                {copySuccess ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
              </Button>
            </TooltipTrigger>
            <TooltipContent side="top" className="flex items-center gap-2">
              <span>{selectedCount > 1 ? `复制（${selectedCount} 项）` : '复制'}</span>
              {showHotkeys && <Kbd>C</Kbd>}
            </TooltipContent>
          </Tooltip>

          {/* 后端收藏功能尚未实装：暂时隐藏入口（保留逻辑，后续直接开关启用）。 */}
          {FAVORITES_UI_ENABLED && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  size="sm"
                  variant="ghost"
                  className={cn(
                    'h-8 w-8 p-0 rounded-full transition-all duration-200',
                    favoriteIntent === 'unfavorite'
                      ? 'text-amber-500 hover:bg-amber-500/10 hover:text-amber-500'
                      : 'hover:bg-amber-500/10 hover:text-amber-500'
                  )}
                  onClick={onToggleFavorite}
                >
                  <Star
                    className={cn('h-4 w-4', favoriteIntent === 'unfavorite' && 'fill-current')}
                  />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="top" className="flex items-center gap-2">
                <span>
                  {selectedCount > 1 ? `${favoriteTitle}（${selectedCount} 项）` : favoriteTitle}
                </span>
                {showHotkeys && <Kbd>S</Kbd>}
              </TooltipContent>
            </Tooltip>
          )}

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size="sm"
                variant="ghost"
                className="h-8 w-8 p-0 rounded-full text-destructive hover:bg-destructive/10 hover:text-destructive transition-all duration-200"
                onClick={onDelete}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="top" className="flex items-center gap-2">
              <span>{selectedCount > 1 ? `删除（${selectedCount} 项）` : '删除'}</span>
              {showHotkeys && <Kbd>D</Kbd>}
            </TooltipContent>
          </Tooltip>

          <div className="w-px h-5 bg-border/50 mx-1" />

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size="sm"
                variant="ghost"
                className="h-8 w-8 p-0 rounded-full hover:bg-muted transition-all duration-200"
                aria-label="取消选择"
                onClick={onClearSelection}
              >
                <X className="h-4 w-4 text-muted-foreground" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="top" className="flex items-center gap-2">
              <span>取消选择</span>
              {showHotkeys && <Kbd>Esc</Kbd>}
            </TooltipContent>
          </Tooltip>
        </div>
      </div>
    </TooltipProvider>
  )
}

export default ClipboardSelectionActionBar
