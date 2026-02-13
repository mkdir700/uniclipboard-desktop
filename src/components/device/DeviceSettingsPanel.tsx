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
      editable: false,
    },
    {
      key: 'syncText',
      defaultChecked: true,
      editable: true,
      mvp: true,
    },
    {
      key: 'syncImage',
      defaultChecked: true,
      editable: false,
    },
    {
      key: 'syncFile',
      defaultChecked: false,
      editable: false,
    },
  ]

  const permissionItems = [
    { key: 'readClipboard', checked: true, editable: false },
    { key: 'writeClipboard', checked: true, editable: false },
    { key: 'historyAccess', checked: true, editable: false },
    { key: 'fileTransfer', checked: false, editable: false },
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
            disabled
            className="text-xs px-2 py-1 rounded-md text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          >
            {t('devices.settings.sync.restoreDefaults')}
          </button>
        </div>

        <div className="divide-y divide-border/40">
          {syncRules.map(rule => (
            <div
              key={rule.key}
              className={`flex items-center justify-between py-3 px-1 ${
                rule.editable ? '' : 'bg-muted/30 text-muted-foreground'
              }`}
            >
              <div className="pr-4">
                <div className="flex items-center gap-2">
                  <h5
                    className={`text-sm font-medium ${
                      rule.editable ? 'text-foreground' : 'text-muted-foreground'
                    }`}
                  >
                    {t(`devices.settings.sync.rules.${rule.key}.title`)}
                  </h5>
                  {rule.mvp && (
                    <span className="text-[10px] leading-none rounded px-1.5 py-1 bg-primary/10 text-primary border border-primary/20">
                      {t('devices.settings.badges.mvp')}
                    </span>
                  )}
                  {!rule.editable && (
                    <span className="text-[10px] leading-none rounded px-1.5 py-1 bg-muted text-muted-foreground">
                      {t('devices.settings.readOnly')}
                    </span>
                  )}
                </div>
                <p
                  className={`text-xs mt-0.5 ${
                    rule.editable ? 'text-muted-foreground' : 'text-muted-foreground/80'
                  }`}
                >
                  {t(`devices.settings.sync.rules.${rule.key}.description`)}
                </p>
              </div>
              <label
                className={`flex items-center shrink-0 ${
                  rule.editable ? 'cursor-pointer' : 'cursor-not-allowed opacity-70'
                }`}
              >
                <div className="relative">
                  <input
                    type="checkbox"
                    className="sr-only peer"
                    defaultChecked={rule.defaultChecked}
                    disabled={!rule.editable}
                  />
                  <div
                    className={`block w-9 h-5 rounded-full transition-colors ${
                      rule.editable
                        ? 'bg-muted peer-checked:bg-primary'
                        : 'bg-muted peer-checked:bg-muted-foreground/40'
                    }`}
                  ></div>
                  <div
                    className={`absolute left-1 top-1 w-3 h-3 rounded-full transition-transform transform peer-checked:translate-x-4 ${
                      rule.editable ? 'bg-white' : 'bg-muted-foreground/40'
                    }`}
                  ></div>
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
            <div
              key={perm.key}
              className={`flex items-center justify-between py-3 px-1 ${
                perm.editable ? '' : 'bg-muted/30 text-muted-foreground'
              }`}
            >
              <div className="flex items-center gap-2">
                <span
                  className={`text-sm font-medium ${
                    perm.editable ? 'text-foreground' : 'text-muted-foreground'
                  }`}
                >
                  {t(`devices.settings.permissions.items.${perm.key}`)}
                </span>
                {!perm.editable && (
                  <span className="text-[10px] leading-none rounded px-1.5 py-1 bg-muted text-muted-foreground">
                    {t('devices.settings.readOnly')}
                  </span>
                )}
              </div>
              <label
                className={`flex items-center shrink-0 ${
                  perm.editable ? 'cursor-pointer' : 'cursor-not-allowed opacity-70'
                }`}
              >
                <div className="relative">
                  <input
                    type="checkbox"
                    className="sr-only peer"
                    defaultChecked={perm.checked}
                    disabled={!perm.editable}
                  />
                  <div
                    className={`block w-9 h-5 rounded-full transition-colors ${
                      perm.editable
                        ? 'bg-muted peer-checked:bg-primary'
                        : 'bg-muted peer-checked:bg-muted-foreground/40'
                    }`}
                  ></div>
                  <div
                    className={`absolute left-1 top-1 w-3 h-3 rounded-full transition-transform transform peer-checked:translate-x-4 ${
                      perm.editable ? 'bg-white' : 'bg-muted-foreground/40'
                    }`}
                  ></div>
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
