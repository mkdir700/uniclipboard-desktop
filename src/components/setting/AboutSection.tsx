import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { SettingSectionHeader } from './SettingSectionHeader'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Card, CardContent } from '@/components/ui/card'
import { Switch } from '@/components/ui/switch'
import { toast } from '@/components/ui/toast'
import { useSetting } from '@/hooks/useSetting'
import { useUpdate } from '@/hooks/useUpdate'

const AboutSection: React.FC = () => {
  const { t } = useTranslation()
  const { setting, loading: settingLoading, updateGeneralSetting } = useSetting()
  const { updateInfo, isCheckingUpdate, checkForUpdates } = useUpdate()
  const [autoCheckUpdate, setAutoCheckUpdate] = useState(true)
  const [updateDialogOpen, setUpdateDialogOpen] = useState(false)
  const [isInstallingUpdate, setIsInstallingUpdate] = useState(false)
  const [saving, setSaving] = useState(false)
  const isBusy = settingLoading || saving

  useEffect(() => {
    if (!setting?.general) return
    setAutoCheckUpdate(setting.general.auto_check_update)
  }, [setting])

  const handleAutoCheckUpdateChange = async (checked: boolean) => {
    const previous = autoCheckUpdate
    try {
      setSaving(true)
      setAutoCheckUpdate(checked)
      await updateGeneralSetting({ auto_check_update: checked })
    } catch (error) {
      console.error('更改自动检查更新状态失败:', error)
      setAutoCheckUpdate(previous)
    } finally {
      setSaving(false)
    }
  }

  const handleCheckUpdate = async () => {
    try {
      const update = await checkForUpdates()
      if (update) {
        setUpdateDialogOpen(true)
      } else {
        toast.success(t('update.noUpdate'))
      }
    } catch (error) {
      console.error('检查更新失败:', error)
      toast.error(t('update.checkFailed'))
    }
  }

  const handleInstallUpdate = async () => {
    if (!updateInfo || isInstallingUpdate) return
    setIsInstallingUpdate(true)

    try {
      await updateInfo.downloadAndInstall()
      await updateInfo.close()
      setUpdateDialogOpen(false)
    } catch (error) {
      console.error('更新失败:', error)
      toast.error(t('update.installFailed'))
    } finally {
      setIsInstallingUpdate(false)
    }
  }

  return (
    <Card>
      <SettingSectionHeader title={t('settings.sections.about.appName')} />
      <CardContent className="pt-0">
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center">
            <div className="h-12 w-12 rounded-xl bg-gradient-to-br from-primary to-primary/60 flex items-center justify-center shadow-lg shadow-primary/20">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-7 w-7 text-primary-foreground"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <title>{t('settings.sections.about.appName')}</title>
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
                />
              </svg>
            </div>
            <div className="ml-4 space-y-0.5">
              <h4 className="text-lg font-medium">{t('settings.sections.about.appName')}</h4>
              <p className="text-sm text-muted-foreground">
                {t('settings.sections.about.version')}
              </p>
            </div>
          </div>
          <button
            type="button"
            className="px-4 py-2 bg-secondary hover:bg-secondary/80 text-sm font-medium transition duration-200 rounded-lg"
            onClick={handleCheckUpdate}
            disabled={isBusy || isCheckingUpdate}
          >
            {t('settings.sections.about.checkUpdate')}
          </button>
        </div>

        <div className="flex items-center justify-between py-2">
          <div className="space-y-0.5">
            <h4 className="text-sm font-medium">
              {t('settings.sections.about.autoCheckUpdate.label')}
            </h4>
            <p className="text-xs text-muted-foreground">
              {t('settings.sections.about.autoCheckUpdate.description')}
            </p>
          </div>
          <Switch
            checked={autoCheckUpdate}
            onCheckedChange={handleAutoCheckUpdateChange}
            disabled={isBusy}
          />
        </div>

        <div className="space-y-4 pt-4 border-t border-border/50">
          <p className="text-sm text-muted-foreground">{t('settings.sections.about.copyright')}</p>
          <div className="flex space-x-6 text-sm">
            <a
              href="https://github.com/UniClipboard/UniClipboard"
              className="text-primary hover:text-primary/80 transition-colors"
              target="_blank"
              rel="noreferrer"
            >
              {t('settings.sections.about.links.privacyPolicy')}
            </a>
            <a
              href="https://github.com/UniClipboard/UniClipboard"
              className="text-primary hover:text-primary/80 transition-colors"
              target="_blank"
              rel="noreferrer"
            >
              {t('settings.sections.about.links.termsOfService')}
            </a>
            <a
              href="https://github.com/UniClipboard/UniClipboard"
              className="text-primary hover:text-primary/80 transition-colors"
              target="_blank"
              rel="noreferrer"
            >
              {t('settings.sections.about.links.helpCenter')}
            </a>
          </div>
        </div>
      </CardContent>
      <AlertDialog open={updateDialogOpen} onOpenChange={setUpdateDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('update.title')}</AlertDialogTitle>
            <AlertDialogDescription asChild className="space-y-3">
              <div>
                <div className="space-y-1">
                  <div className="flex items-center justify-between text-muted-foreground">
                    <span>{t('update.currentVersion')}</span>
                    <span className="text-foreground">{updateInfo?.currentVersion ?? '-'}</span>
                  </div>
                  <div className="flex items-center justify-between text-muted-foreground">
                    <span>{t('update.latestVersion')}</span>
                    <span className="text-foreground">{updateInfo?.version ?? '-'}</span>
                  </div>
                </div>
                <div className="space-y-2">
                  <div className="text-sm font-medium text-foreground">
                    {t('update.releaseNotes')}
                  </div>
                  <div className="max-h-48 overflow-auto rounded-md border border-border/60 bg-muted/30 px-3 py-2 text-sm text-muted-foreground whitespace-pre-wrap">
                    {updateInfo?.body?.trim() ? updateInfo.body : t('update.noNotes')}
                  </div>
                </div>
              </div>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isInstallingUpdate}>{t('update.later')}</AlertDialogCancel>
            <AlertDialogAction
              onClick={event => {
                event.preventDefault()
                handleInstallUpdate()
              }}
              disabled={isInstallingUpdate}
            >
              {t('update.updateNow')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </Card>
  )
}

export default AboutSection
