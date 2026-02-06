import { motion } from 'framer-motion'
import { AlertCircle, Check, X, Loader2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { PairingConfirmStepProps } from './types'
import { Button } from '@/components/ui/button'

export default function PairingConfirmStep({
  shortCode,
  peerFingerprint,
  onConfirm,
  onCancel,
  error,
  loading,
}: PairingConfirmStepProps) {
  const { t } = useTranslation(undefined, { keyPrefix: 'setup.pairingConfirm' })

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.98 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.98 }}
      className="w-full"
    >
      <div className="mb-10 text-center">
        <h1 className="text-2xl font-semibold tracking-tight text-foreground">{t('title')}</h1>
        <p className="mt-2 text-muted-foreground">{t('subtitle')}</p>
      </div>

      <div className="mb-10 text-center">
        <div className="text-5xl font-mono font-semibold tracking-widest text-primary">
          {shortCode}
        </div>
        {peerFingerprint && (
          <div className="mt-6 pt-6 border-t border-border/30">
            <div className="text-xs text-muted-foreground mb-1">{t('peerFingerprint')}</div>
            <div className="font-mono text-xs break-all opacity-70">{peerFingerprint}</div>
          </div>
        )}
      </div>

      {error && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="mb-6 flex items-center justify-center gap-2 text-sm text-destructive"
        >
          <AlertCircle className="h-4 w-4 shrink-0" />
          {error === 'PairingRejected' ? t('errors.rejected') : t('errors.generic')}
        </motion.div>
      )}

      <div className="flex justify-center gap-4">
        <Button variant="outline" onClick={onCancel} disabled={loading}>
          <X className="mr-2 h-4 w-4" />
          {t('actions.cancel')}
        </Button>
        <Button onClick={onConfirm} disabled={loading}>
          {loading ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              {t('actions.confirming')}
            </>
          ) : (
            <>
              <Check className="mr-2 h-4 w-4" />
              {t('actions.confirm')}
            </>
          )}
        </Button>
      </div>
    </motion.div>
  )
}
