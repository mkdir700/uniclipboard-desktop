import { Eye, EyeOff } from 'lucide-react'
import React, { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { setEncryptionPassword, getEncryptionPassword } from '@/api/security'
import { Switch, Input } from '@/components/ui'
import { Card, CardContent } from '@/components/ui/card'
import { useSetting } from '@/hooks/useSetting'

const SecuritySection: React.FC = () => {
  const { t } = useTranslation()
  const { setting, error, updateSecuritySetting } = useSetting()

  // Local state
  const [endToEndEncryption, setEndToEndEncryption] = useState(true)
  const [autoUnlockEnabled, setAutoUnlockEnabled] = useState(false)
  const [encryptionPassword, setEncryptionPasswordInput] = useState('')
  const [showPassword, setShowPassword] = useState(false)

  // Debounce save password
  useEffect(() => {
    const timer = setTimeout(() => {
      setEncryptionPassword(encryptionPassword)
    }, 500)
    return () => clearTimeout(timer)
  }, [encryptionPassword])

  // Update local state when settings are loaded
  useEffect(() => {
    if (setting) {
      setEndToEndEncryption(setting.security.encryption_enabled)
      setAutoUnlockEnabled(setting.security.auto_unlock_enabled)
      getEncryptionPassword().then(password => {
        setEncryptionPasswordInput(password || '')
      })
    }
  }, [setting])

  // Handle end-to-end encryption toggle change
  const handleEndToEndEncryptionChange = (checked: boolean) => {
    const newValue = checked
    setEndToEndEncryption(newValue)
    updateSecuritySetting({ encryption_enabled: newValue })
  }

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
        {/* End-to-end encryption */}
        <div className="flex items-center justify-between py-2">
          <div className="space-y-0.5">
            <h4 className="text-sm font-medium">
              {t('settings.sections.security.endToEndEncryption.label')}
            </h4>
            <p className="text-xs text-muted-foreground">
              {t('settings.sections.security.endToEndEncryption.description')}
            </p>
          </div>
          <Switch checked={endToEndEncryption} onCheckedChange={handleEndToEndEncryptionChange} />
        </div>

        {/* Encryption password */}
        <div className="flex items-center justify-between gap-4 py-2">
          <div className="space-y-0.5">
            <h4 className="text-sm font-medium">
              {t('settings.sections.security.encryptionPassword.label')}
            </h4>
            <p className="text-xs text-muted-foreground">
              {t('settings.sections.security.encryptionPassword.description')}
            </p>
          </div>
          <div className="relative flex items-center">
            <Input
              type={showPassword ? 'text' : 'password'}
              value={encryptionPassword}
              onChange={e => setEncryptionPasswordInput(e.target.value)}
              placeholder={t('settings.sections.security.encryptionPassword.placeholder')}
              className="w-64 pr-10"
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-2 text-muted-foreground hover:text-foreground transition-colors"
            >
              {showPassword ? <EyeOff size={18} /> : <Eye size={18} />}
            </button>
          </div>
        </div>

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
