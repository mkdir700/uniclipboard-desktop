import { ChevronDown, ChevronUp, File, ExternalLink } from 'lucide-react'
import React, { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  ClipboardTextItem,
  ClipboardImageItem,
  ClipboardLinkItem,
  ClipboardCodeItem,
  ClipboardFileItem,
} from '@/api/clipboardItems'
import { cn } from '@/lib/utils'
import { formatFileSize } from '@/utils'

interface ClipboardItemProps {
  index: number
  type: 'text' | 'image' | 'link' | 'code' | 'file' | 'unknown'
  time: string
  device?: string
  content:
    | ClipboardTextItem
    | ClipboardImageItem
    | ClipboardLinkItem
    | ClipboardCodeItem
    | ClipboardFileItem
    | null
  isSelected?: boolean
  onSelect?: (event: React.MouseEvent<HTMLDivElement>) => void
  fileSize?: number
}

const ClipboardItem: React.FC<ClipboardItemProps> = ({
  index,
  type,
  time,
  content,
  isSelected = false,
  onSelect,
  fileSize,
}) => {
  const { t } = useTranslation()
  const [isExpanded, setIsExpanded] = useState(false)

  // Calculate character count or size info
  const getSizeInfo = (): string => {
    if (!content) return ''
    switch (type) {
      case 'text':
        return `${(content as ClipboardTextItem).display_text.length} ${t('clipboard.item.characters')}`
      case 'link':
        return t('clipboard.item.link')
      case 'code':
        return `${(content as ClipboardCodeItem).code.length} ${t('clipboard.item.characters')}`
      case 'file':
        return formatFileSize(fileSize)
      case 'image':
        // Note: Use actual dimensions if available in API, otherwise placeholder or remove
        return t('clipboard.item.image')
      default:
        return ''
    }
  }

  const renderContent = () => {
    switch (type) {
      case 'text':
        return (
          <p
            className={cn(
              'whitespace-pre-wrap font-mono text-sm leading-relaxed text-foreground/90 wrap-break-word',
              !isExpanded && 'line-clamp-5'
            )}
          >
            {(content as ClipboardTextItem).display_text}
          </p>
        )
      case 'image':
        return (
          <div className="flex justify-center bg-black/20 rounded-lg overflow-hidden py-4">
            <img
              src={(content as ClipboardImageItem).thumbnail}
              className={cn(
                'w-auto object-contain rounded-md shadow-sm transition-all duration-300',
                isExpanded ? 'max-h-[500px]' : 'h-32'
              )}
              alt={t('clipboard.item.altText.clipboardImage')}
              loading="lazy"
            />
          </div>
        )
      case 'link': {
        const url = (content as ClipboardLinkItem).url
        return (
          <div className="flex flex-col gap-1">
            <a
              href={url}
              target="_blank"
              rel="noreferrer"
              className="text-primary font-medium hover:underline break-all text-sm leading-relaxed flex items-center gap-2"
              onClick={e => e.stopPropagation()}
            >
              <ExternalLink size={14} />
              {url}
            </a>
          </div>
        )
      }
      case 'code':
        return (
          <div className="bg-muted/30 p-3 rounded-lg border border-border/30 overflow-hidden font-mono text-xs">
            <pre
              className={cn(
                'whitespace-pre-wrap break-all text-foreground/80',
                !isExpanded && 'line-clamp-6'
              )}
            >
              {(content as ClipboardCodeItem).code}
            </pre>
          </div>
        )
      case 'file': {
        const fileNames = (content as ClipboardFileItem).file_names
        return (
          <div className="flex flex-col gap-2">
            {fileNames.map((name, i) => (
              <div key={i} className="flex items-center gap-2 text-sm text-foreground/80">
                <File size={16} className="text-muted-foreground" />
                <span className="truncate">{name}</span>
              </div>
            ))}
          </div>
        )
      }
      default:
        return <p className="text-muted-foreground text-sm">{t('clipboard.item.unknownContent')}</p>
    }
  }

  return (
    <div
      className={cn(
        'group relative flex flex-col border-b border-border/40 transition-all duration-300 select-none',
        isSelected
          ? 'bg-primary/5 border-l-4 border-l-primary'
          : 'hover:bg-muted/20 border-l-4 border-l-transparent hover:border-l-primary/30'
      )}
      onClick={onSelect}
    >
      {/* Main Content Area */}
      <div className="p-4">{renderContent()}</div>

      {/* Footer Area */}
      <div className="flex items-center justify-between px-4 pb-2 pt-1 text-xs text-muted-foreground/60 select-none">
        {/* Left: Time */}
        <div className="min-w-20">{time}</div>

        {/* Center: Expand Button (Visible if expandable content, or always visible logic) */}
        <div
          className="flex items-center gap-1 cursor-pointer hover:text-foreground transition-colors px-2 py-1 rounded-md hover:bg-muted/50"
          onClick={e => {
            e.stopPropagation()
            setIsExpanded(!isExpanded)
          }}
        >
          {isExpanded ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
          <span>{isExpanded ? t('clipboard.item.collapse') : t('clipboard.item.expand')}</span>
        </div>

        {/* Right: Stats & Index */}
        <div className="flex items-center gap-4 min-w-20 justify-end">
          <span>{getSizeInfo()}</span>
          <span className="font-mono text-muted-foreground/40">{index}</span>
        </div>
      </div>
    </div>
  )
}

export default ClipboardItem
