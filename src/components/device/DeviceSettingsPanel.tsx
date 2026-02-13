import React from 'react'
import { useTranslation } from 'react-i18next'

interface DeviceSettingsPanelProps {
  deviceId: string
  deviceName: string
}

const DeviceSettingsPanel: React.FC<DeviceSettingsPanelProps> = () => {
  const { t } = useTranslation()

  const syncRules = [
    {
      key: 'autoSync',
      defaultChecked: true,
    },
    {
      key: 'syncText',
      defaultChecked: true,
    },
    {
      key: 'syncImage',
      defaultChecked: true,
    },
    {
      key: 'syncFile',
      defaultChecked: false,
    },
  ]

  const permissionItems = [
    { key: 'readClipboard', checked: true },
    { key: 'writeClipboard', checked: true },
    { key: 'historyAccess', checked: true },
    { key: 'fileTransfer', checked: false },
  ]

  return (
    <div className="space-y-6">
      <div>
        <div className="flex items-center justify-between mb-2 px-1">
          <h4 className="text-sm font-medium text-muted-foreground uppercase tracking-wider">
            {t('devices.settings.sync.title')}
          </h4>
          <button
            type="button"
            className="text-xs px-2 py-1 rounded-md text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          >
            {t('devices.settings.sync.restoreDefaults')}
          </button>
        </div>

        <div className="divide-y divide-border/40">
          {syncRules.map(rule => (
            <div key={rule.key} className="flex items-center justify-between py-3 px-1">
              <div className="pr-4">
                <h5 className="text-sm font-medium text-foreground">
                  {t(`devices.settings.sync.rules.${rule.key}.title`)}
                </h5>
                <p className="text-xs text-muted-foreground mt-0.5">
                  {t(`devices.settings.sync.rules.${rule.key}.description`)}
                </p>
              </div>
              <label className="flex items-center cursor-pointer shrink-0">
                <div className="relative">
                  <input
                    type="checkbox"
                    className="sr-only peer"
                    defaultChecked={rule.defaultChecked}
                  />
                  <div className="block bg-muted w-9 h-5 rounded-full peer-checked:bg-primary transition-colors"></div>
                  <div className="absolute left-1 top-1 bg-white w-3 h-3 rounded-full transition-transform transform peer-checked:translate-x-4"></div>
                </div>
              </label>
            </div>
          ))}
        </div>
      </div>

      <div>
        <div className="mb-2 px-1">
          <h4 className="text-sm font-medium text-muted-foreground uppercase tracking-wider">
            {t('devices.settings.permissions.title')}
          </h4>
        </div>

        <div className="divide-y divide-border/40">
          {permissionItems.map(perm => (
            <div key={perm.key} className="flex items-center justify-between py-3 px-1">
              <span className="text-sm font-medium text-foreground">
                {t(`devices.settings.permissions.items.${perm.key}`)}
              </span>
              <label className="flex items-center cursor-pointer shrink-0">
                <div className="relative">
                  <input type="checkbox" className="sr-only peer" defaultChecked={perm.checked} />
                  <div className="block bg-muted w-9 h-5 rounded-full peer-checked:bg-primary transition-colors"></div>
                  <div className="absolute left-1 top-1 bg-white w-3 h-3 rounded-full transition-transform transform peer-checked:translate-x-4"></div>
                </div>
              </label>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

export default DeviceSettingsPanel
