import { motion } from 'framer-motion'
import { AlertCircle, Eye, EyeOff, Loader2, ArrowLeft } from 'lucide-react'
import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { CreatePassphraseStepProps } from '@/pages/onboarding/types'

export default function CreatePassphraseStep({
  onSubmit,
  onBack,
  error,
  loading,
}: CreatePassphraseStepProps) {
  const { t } = useTranslation(undefined, { keyPrefix: 'onboarding.createPassphrase' })
  const { t: tCommon } = useTranslation(undefined, { keyPrefix: 'onboarding.common' })

  const [pass1, setPass1] = useState('')
  const [pass2, setPass2] = useState('')
  const [showPass1, setShowPass1] = useState(false)
  const [showPass2, setShowPass2] = useState(false)
  const [localError, setLocalError] = useState<string | null>(null)

  useEffect(() => {
    if (!error) {
      setLocalError(null)
      return
    }

    if (error === 'PassphraseMismatch') {
      setLocalError(t('errors.mismatch'))
    } else if (typeof error === 'object' && 'PassphraseTooShort' in error) {
      setLocalError(t('errors.tooShort', { minLen: error.PassphraseTooShort.min_len }))
    } else if (error === 'PassphraseEmpty') {
      setLocalError(t('errors.empty'))
    } else {
      setLocalError(t('errors.generic'))
    }
  }, [error, t])

  const handleSubmit = () => {
    if (!pass1) {
      setLocalError(t('errors.empty'))
      return
    }
    if (pass1 !== pass2) {
      setLocalError(t('errors.mismatch'))
      return
    }
    onSubmit(pass1, pass2)
  }

  return (
    <motion.div
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: -20 }}
      className="w-full"
    >
      <button
        onClick={onBack}
        className="mb-8 flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
      >
        <ArrowLeft className="h-4 w-4" />
        {tCommon('back')}
      </button>

      <div className="mb-10">
        <h1 className="text-2xl font-semibold tracking-tight text-foreground">{t('title')}</h1>
        <p className="mt-2 text-muted-foreground">{t('subtitle')}</p>
      </div>

      <div className="space-y-6">
        <div className="space-y-4">
          <Label htmlFor="pass1" className="block">
            {t('labels.pass1')}
          </Label>
          <div className="relative">
            <Input
              id="pass1"
              type={showPass1 ? 'text' : 'password'}
              value={pass1}
              onChange={e => setPass1(e.target.value)}
              disabled={loading}
              className="pr-10"
              placeholder={t('placeholders.pass1')}
            />
            <button
              type="button"
              onClick={() => setShowPass1(!showPass1)}
              className="absolute right-0 top-0 flex h-full items-center px-3 text-muted-foreground transition-colors hover:text-foreground"
            >
              {showPass1 ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </button>
          </div>
        </div>

        <div className="space-y-4">
          <Label htmlFor="pass2" className="block">
            {t('labels.pass2')}
          </Label>
          <div className="relative">
            <Input
              id="pass2"
              type={showPass2 ? 'text' : 'password'}
              value={pass2}
              onChange={e => setPass2(e.target.value)}
              disabled={loading}
              className="pr-10"
              placeholder={t('placeholders.pass2')}
              onKeyDown={e => e.key === 'Enter' && handleSubmit()}
            />
            <button
              type="button"
              onClick={() => setShowPass2(!showPass2)}
              className="absolute right-0 top-0 flex h-full items-center px-3 text-muted-foreground transition-colors hover:text-foreground"
            >
              {showPass2 ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </button>
          </div>
        </div>

        {localError && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="flex items-center gap-2 text-sm text-destructive"
          >
            <AlertCircle className="h-4 w-4 shrink-0" />
            {localError}
          </motion.div>
        )}
      </div>

      <div className="mt-10 flex items-center gap-4">
        <Button onClick={handleSubmit} disabled={loading} className="min-w-32">
          {loading ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              {t('actions.creating')}
            </>
          ) : (
            t('actions.submit')
          )}
        </Button>
      </div>

      <p className="mt-6 text-xs text-muted-foreground">{t('hint')}</p>
    </motion.div>
  )
}
