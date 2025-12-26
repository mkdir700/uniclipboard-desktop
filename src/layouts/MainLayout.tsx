import React, { ReactNode } from 'react'
import { Sidebar } from '@/components'

interface MainLayoutProps {
  children: ReactNode
}

const MainLayout: React.FC<MainLayoutProps> = ({ children }) => {
  return (
    <div className="h-screen flex overflow-hidden bg-background text-foreground transition-colors duration-200">
      {/* 侧边栏导航 */}
      <Sidebar />

      {/* 主内容区域 */}
      <main className="flex-1 flex flex-col overflow-hidden relative">
        {/* Window Drag Region handled by TitleBar */}
        {children}
      </main>
    </div>
  )
}

export default MainLayout
