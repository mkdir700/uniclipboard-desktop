import { motion } from 'framer-motion'
import { CheckCircle2, ArrowRight } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { SetupDoneStepProps } from '@/pages/onboarding/types'

export default function SetupDoneStep({ onComplete, loading }: SetupDoneStepProps) {
  const { t } = useTranslation(undefined, { keyPrefix: 'onboarding.done' })

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.98 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.98 }}
      className="w-full"
    >
      <div className="flex flex-col items-center text-center">
        <div className="mb-8 flex h-20 w-20 items-center justify-center text-green-500">
          <CheckCircle2 className="h-16 w-16" />
        </div>

        <h1 className="text-2xl font-semibold tracking-tight text-foreground">{t('title')}</h1>
        <p className="mt-2 max-w-sm text-muted-foreground">{t('subtitle')}</p>

        <Button onClick={onComplete} disabled={loading} className="mt-10 min-w-40">
          {t('actions.enter')}
          <ArrowRight className="ml-2 h-4 w-4" />
        </Button>
      </div>
    </motion.div>
  )
}
