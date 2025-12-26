import React, { ReactNode } from 'react'
import { SettingHeader } from '../components'
import { CategoryItem } from '../components/setting/SettingHeader'
import { MainLayout } from './index'

interface SettingsLayoutProps {
  children: ReactNode
  onCategoryClick: (category: string) => void
  activeCategory: string
  categories: CategoryItem[]
}

const SettingsLayout: React.FC<SettingsLayoutProps> = ({
  children,
  onCategoryClick,
  activeCategory,
  categories,
}) => {
  return (
    <MainLayout>
      {/* 顶部标题栏 */}
      <SettingHeader
        onCategoryClick={onCategoryClick}
        activeCategory={activeCategory}
        categories={categories}
      />

      {/* 主内容区域 */}
      <div className="flex-1 overflow-hidden">
        <div className="h-full px-4 py-3 overflow-auto hide-scrollbar">{children}</div>
      </div>

      {/* 底部操作栏 */}
      {/* <SettingFooter /> */}
    </MainLayout>
  )
}

export default SettingsLayout
