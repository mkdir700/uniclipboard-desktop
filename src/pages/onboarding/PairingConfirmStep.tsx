import { motion } from 'framer-motion'
import { AlertCircle, Check, X, Loader2 } from 'lucide-react'
import { PairingConfirmStepProps } from './types'
import { Button } from '@/components/ui/button'

export default function PairingConfirmStep({
  shortCode,
  sessionId,
  peerFingerprint,
  onConfirm,
  onCancel,
  error,
  loading,
}: PairingConfirmStepProps) {
  return (
    <motion.div
      initial={{ scale: 0.9, opacity: 0 }}
      animate={{ scale: 1, opacity: 1 }}
      exit={{ scale: 0.9, opacity: 0 }}
      className="w-full max-w-md mx-auto text-center"
    >
      <h1 className="text-2xl font-bold text-foreground mb-2">确认配对</h1>
      <p className="text-muted-foreground text-sm mb-8">请确认另一台设备上显示的配对码是否一致</p>

      <div className="bg-muted/30 rounded-2xl p-8 mb-8 border border-border/50">
        <div className="text-4xl font-mono font-bold tracking-widest text-primary mb-2">
          {shortCode}
        </div>
        <div className="text-xs text-muted-foreground font-mono opacity-70">
          Session ID: {sessionId.substring(0, 8)}...
        </div>
        {peerFingerprint && (
          <div className="mt-4 pt-4 border-t border-border/30">
            <div className="text-xs text-muted-foreground mb-1">对方设备指纹</div>
            <div className="text-xs font-mono break-all opacity-80">{peerFingerprint}</div>
          </div>
        )}
      </div>

      {error && (
        <motion.div
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          className="mb-6 p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-sm flex items-center justify-center gap-2"
        >
          <AlertCircle className="w-4 h-4 shrink-0" />
          {error === 'PairingRejected' ? '对方拒绝了配对请求' : '配对失败，请重试'}
        </motion.div>
      )}

      <div className="grid grid-cols-2 gap-4">
        <Button variant="outline" onClick={onCancel} disabled={loading} className="h-12">
          <X className="w-4 h-4 mr-2" />
          取消
        </Button>
        <Button onClick={onConfirm} disabled={loading} className="h-12">
          {loading ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              确认中...
            </>
          ) : (
            <>
              <Check className="w-4 h-4 mr-2" />
              确认配对
            </>
          )}
        </Button>
      </div>
    </motion.div>
  )
}
