import { useTranslation } from 'react-i18next'
import { SettingSectionHeader } from './SettingSectionHeader'
import { Card, CardContent } from '@/components/ui/card'

/**
 * Storage Section Placeholder
 *
 * 存储设置功能在新架构中尚未实现。
 * 后端 Settings 结构中不存在 storage 字段。
 * 此组件作为占位符，待后端实现后替换。
 *
 * 注意：后端有 RetentionPolicy 结构，可用于实现类似功能。
 */
const StorageSection: React.FC = () => {
  const { t } = useTranslation()

  return (
    <Card>
      <SettingSectionHeader title={t('settings.categories.storage')} />
      <CardContent className="pt-0">
        <div className="text-center py-8">
          <p className="text-sm text-muted-foreground">
            {t('settings.sections.storage.placeholder') || '存储设置功能暂未实现'}
          </p>
          <p className="text-xs text-muted-foreground mt-2">
            Storage settings are not yet available in the new architecture.
          </p>
        </div>
      </CardContent>
    </Card>
  )
}

export default StorageSection
