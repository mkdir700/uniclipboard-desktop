import { afterEach, beforeAll, describe, expect, it } from 'vitest'
import i18n from '@/i18n'

describe('onboarding i18n keys', () => {
  let initialLanguage: string

  const ensureI18nInitialized = async () => {
    if (i18n.isInitialized) return

    await new Promise<void>(resolve => {
      const handler = () => {
        i18n.off('initialized', handler)
        resolve()
      }
      i18n.on('initialized', handler)
    })
  }

  beforeAll(async () => {
    await ensureI18nInitialized()
    initialLanguage = i18n.language
  })

  afterEach(async () => {
    await i18n.changeLanguage(initialLanguage)
  })

  it('resolves zh-CN onboarding.welcome.title', async () => {
    await i18n.changeLanguage('zh-CN')
    expect(i18n.t('onboarding.welcome.title')).toBe('欢迎使用 UniClipboard')
    expect(i18n.t('onboarding.preparingDashboard')).toBe('准备您的仪表板...')
  })

  it('resolves en-US onboarding.welcome.title', async () => {
    await i18n.changeLanguage('en-US')
    expect(i18n.t('onboarding.welcome.title')).toBe('Welcome to UniClipboard')
    expect(i18n.t('onboarding.preparingDashboard')).toBe('Preparing your dashboard...')
  })
})
