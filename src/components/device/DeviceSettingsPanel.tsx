import React from 'react'

interface DeviceSettingsPanelProps {
  deviceId: string
  deviceName: string
}

const DeviceSettingsPanel: React.FC<DeviceSettingsPanelProps> = () => {
  return (
    <div className="space-y-6">
      <div>
        <div className="flex items-center justify-between mb-2 px-1">
          <h4 className="text-sm font-medium text-muted-foreground uppercase tracking-wider">
            同步设置
          </h4>
          <button
            type="button"
            className="text-xs px-2 py-1 rounded-md text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          >
            恢复默认
          </button>
        </div>

        <div className="divide-y divide-border/40">
          {[
            {
              title: '自动同步',
              desc: '在设备解锁状态下自动同步剪贴板内容',
              defaultChecked: true,
            },
            { title: '同步文本', desc: '允许同步文本内容', defaultChecked: true },
            {
              title: '同步图片',
              desc: '允许同步图片内容 (可能会消耗更多流量)',
              defaultChecked: true,
            },
            { title: '同步文件', desc: '允许同步文件内容 (最大10MB)', defaultChecked: false },
          ].map(rule => (
            <div key={rule.title} className="flex items-center justify-between py-3 px-1">
              <div className="pr-4">
                <h5 className="text-sm font-medium text-foreground">{rule.title}</h5>
                <p className="text-xs text-muted-foreground mt-0.5">{rule.desc}</p>
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
            访问权限
          </h4>
        </div>

        <div className="divide-y divide-border/40">
          {[
            { label: '读取剪贴板', checked: true },
            { label: '写入剪贴板', checked: true },
            { label: '访问历史记录', checked: true },
            { label: '传输文件', checked: false },
          ].map(perm => (
            <div key={perm.label} className="flex items-center justify-between py-3 px-1">
              <span className="text-sm font-medium text-foreground">{perm.label}</span>
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
