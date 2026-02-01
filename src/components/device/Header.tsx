import { Plus } from 'lucide-react'
import React from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'

interface HeaderProps {
  addDevice: () => void
}

const Header: React.FC<HeaderProps> = ({ addDevice }) => {
  const { t } = useTranslation()

  return (
    <header
      data-tauri-drag-region
      className="sticky top-0 z-50 shrink-0 pt-6 pb-6 px-8 transition-all duration-300"
    >
      {/* Glass Background */}
      <div
        data-tauri-drag-region
        className="absolute inset-0 bg-background/60 backdrop-blur-xl border-b border-white/5 shadow-sm"
      />

      <div data-tauri-drag-region className="relative z-10">
        {/* Top Row: Title & Action */}
        <div className="flex items-center justify-between">
          <h1 data-tauri-drag-region className="text-2xl font-bold tracking-tight">
            {t('devices.title')}
          </h1>

          <Button
            data-tauri-drag-region="false"
            onClick={addDevice}
            className="bg-primary hover:bg-primary/90 text-primary-foreground shadow-lg shadow-primary/25 rounded-lg px-4 py-2 h-auto text-sm font-medium transition-all duration-300 transform hover:scale-105 active:scale-95"
          >
            <Plus className="h-4 w-4 mr-2" />
            {t('devices.addNew')}
          </Button>
        </div>
      </div>
    </header>
  )
}

export default Header
