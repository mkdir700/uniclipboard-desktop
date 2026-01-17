import { useTranslation } from 'react-i18next'
import { Card, CardContent } from '@/components/ui/card'

/**
 * Network Section Placeholder
 *
 * 网络设置功能在新架构中尚未实现。
 * 后端 Settings 结构中不存在 network 字段。
 * 此组件作为占位符，待后端实现后替换。
 */
const NetworkSection: React.FC = () => {
  const { t } = useTranslation()

  return (
    <Card>
      <CardContent className="pt-6">
        <div className="text-center py-8">
          <p className="text-sm text-muted-foreground">
            {t('settings.sections.network.placeholder') || '网络设置功能暂未实现'}
          </p>
          <p className="text-xs text-muted-foreground mt-2">
            Network settings are not yet available in the new architecture.
          </p>
        </div>
      </CardContent>
    </Card>
  )
}

export default NetworkSection
