import React, { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Switch } from '@/components/ui'
import { Card, CardContent } from '@/components/ui/card'
import { useSetting } from '@/hooks/useSetting'

const SecuritySection: React.FC = () => {
  const { t } = useTranslation()
  const { setting, error, updateSecuritySetting } = useSetting()

  const [autoUnlockEnabled, setAutoUnlockEnabled] = useState(false)

  // Update local state when settings are loaded
  useEffect(() => {
    if (setting) {
      setAutoUnlockEnabled(setting.security.auto_unlock_enabled)
    }
  }, [setting])

  const handleAutoUnlockChange = (checked: boolean) => {
    setAutoUnlockEnabled(checked)
    updateSecuritySetting({ auto_unlock_enabled: checked })
  }

  // Display error message if there is an error
  if (error) {
    return (
      <div className="text-red-500 py-4">
        {t('settings.sections.security.loadError')}: {error}
      </div>
    )
  }

  return (
    <Card>
      <CardContent className="pt-6 space-y-4">
        {/* Auto unlock */}
        <div className="flex items-center justify-between py-2">
          <div className="space-y-0.5">
            <h4 className="text-sm font-medium">
              {t('settings.sections.security.autoUnlock.label')}
            </h4>
            <p className="text-xs text-muted-foreground">
              {t('settings.sections.security.autoUnlock.description')}
            </p>
          </div>
          <Switch checked={autoUnlockEnabled} onCheckedChange={handleAutoUnlockChange} />
        </div>
      </CardContent>
    </Card>
  )
}

export default SecuritySection
