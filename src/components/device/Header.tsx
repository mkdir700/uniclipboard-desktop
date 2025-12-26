import { motion } from 'framer-motion'
import { Plus } from 'lucide-react'
import React from 'react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

export type DeviceTab = 'connected' | 'requests'

interface HeaderProps {
  addDevice: () => void
  activeTab: DeviceTab
  onTabChange: (tab: DeviceTab) => void
}

const Header: React.FC<HeaderProps> = ({ addDevice, activeTab, onTabChange }) => {
  const tabs: { id: DeviceTab; label: string }[] = [
    { id: 'requests', label: '配对请求' },
    { id: 'connected', label: '已连接设备' },
  ]

  return (
    <header
      data-tauri-drag-region
      className="sticky top-0 z-50 pt-6 pb-2 px-8 transition-all duration-300"
    >
      {/* Glass Background */}
      <div
        data-tauri-drag-region
        className="absolute inset-0 bg-background/60 backdrop-blur-xl border-b border-white/5 shadow-sm"
      />

      <div data-tauri-drag-region className="relative z-10 space-y-4">
        {/* Top Row: Title & Action */}
        <div className="flex items-center justify-between">
          <h1 data-tauri-drag-region className="text-2xl font-bold tracking-tight">
            设备管理
          </h1>

          <Button
            data-tauri-drag-region="false"
            onClick={addDevice}
            className="bg-primary hover:bg-primary/90 text-primary-foreground shadow-lg shadow-primary/25 rounded-lg px-4 py-2 h-auto text-sm font-medium transition-all duration-300 transform hover:scale-105 active:scale-95"
          >
            <Plus className="h-4 w-4 mr-2" />
            添加新设备
          </Button>
        </div>

        {/* Tabs Scroll Area */}
        <div className="flex items-center gap-2 overflow-x-auto no-scrollbar pb-2 -mx-8 px-8 mask-linear-fade">
          {tabs.map(tab => {
            const isActive = activeTab === tab.id

            return (
              <motion.button
                data-tauri-drag-region="false"
                key={tab.id}
                onClick={() => onTabChange(tab.id)}
                className={cn(
                  'relative group flex items-center justify-center px-4 py-2 rounded-lg text-sm font-medium whitespace-nowrap transition-all duration-300 outline-none select-none',
                  isActive
                    ? 'text-primary-foreground'
                    : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'
                )}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.96 }}
              >
                {isActive && (
                  <motion.div
                    layoutId="activeDeviceTab"
                    className="absolute inset-0 bg-primary rounded-lg shadow-lg shadow-primary/25"
                    transition={{ type: 'spring', bounce: 0.2, duration: 0.6 }}
                  />
                )}
                <span className="relative z-10">{tab.label}</span>
              </motion.button>
            )
          })}
        </div>
      </div>
    </header>
  )
}

export default Header
