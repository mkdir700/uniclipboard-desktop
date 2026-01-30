import { motion } from 'framer-motion'
import { AlertCircle, Eye, EyeOff, Loader2, ArrowLeft } from 'lucide-react'
import { useState, useEffect } from 'react'
import { JoinVerifyPassphraseStepProps } from './types'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

export default function JoinVerifyPassphraseStep({
  peerId,
  onSubmit,
  onBack,
  onCreateNew,
  error,
  loading,
}: JoinVerifyPassphraseStepProps) {
  const [passphrase, setPassphrase] = useState('')
  const [showPassphrase, setShowPassphrase] = useState(false)
  const [localError, setLocalError] = useState<string | null>(null)
  const [showMismatchHelp, setShowMismatchHelp] = useState(true)

  useEffect(() => {
    if (!error) {
      setLocalError(null)
      setShowMismatchHelp(true)
      return
    }
    setShowMismatchHelp(true)
    if (error === 'PassphraseInvalidOrMismatch') {
      setLocalError(null)
    } else if (error === 'NetworkTimeout') {
      setLocalError('验证超时，请检查网络后重试。')
    } else if (error === 'PeerUnavailable') {
      setLocalError('对方设备不可用，请确认已打开 UniClipboard。')
    } else {
      setLocalError('验证失败，请重试。')
    }
  }, [error])

  const handleSubmit = () => {
    if (!passphrase) {
      setLocalError('请输入加密口令。')
      return
    }
    onSubmit(passphrase)
  }

  return (
    <motion.div
      initial={{ x: 300, opacity: 0 }}
      animate={{ x: 0, opacity: 1 }}
      exit={{ x: -300, opacity: 0 }}
      className="w-full max-w-md mx-auto"
    >
      <div className="mb-8">
        <Button
          variant="ghost"
          size="sm"
          className="pl-0 hover:bg-transparent text-muted-foreground hover:text-foreground"
          onClick={onBack}
        >
          <ArrowLeft className="w-4 h-4 mr-1" />
          返回选择设备
        </Button>
        <h1 className="text-2xl font-bold text-foreground mt-2">输入加密口令</h1>
        <p className="text-muted-foreground text-sm mt-1">
          请输入与你已有设备相同的加密口令，用于加入同一个加密空间并解密剪贴板数据。
        </p>
        <p className="text-xs text-muted-foreground mt-1 font-mono">
          目标设备 ID: {peerId.substring(0, 8)}...
        </p>
      </div>

      <div className="space-y-5 mb-8">
        <div className="space-y-2">
          <Label htmlFor="passphrase">加密口令</Label>
          <div className="relative">
            <Input
              id="passphrase"
              type={showPassphrase ? 'text' : 'password'}
              value={passphrase}
              onChange={e => setPassphrase(e.target.value)}
              disabled={loading}
              className="pr-10"
              placeholder="输入加密口令"
              onKeyDown={e => e.key === 'Enter' && handleSubmit()}
            />
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
              onClick={() => setShowPassphrase(!showPassphrase)}
            >
              {showPassphrase ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
            </Button>
          </div>
        </div>

        {localError && (
          <motion.div
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            className="p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-sm flex items-center gap-2"
          >
            <AlertCircle className="w-4 h-4 shrink-0" />
            {localError}
          </motion.div>
        )}
      </div>

      {error === 'PassphraseInvalidOrMismatch' && showMismatchHelp ? (
        <div className="space-y-3">
          <div className="rounded-lg border border-destructive/30 bg-destructive/10 p-4 text-sm text-destructive">
            <div className="font-semibold">无法加入加密空间</div>
            <p className="text-xs mt-1">
              当前输入的加密口令与已有设备使用的口令不一致。只有使用相同口令的设备才能加入同一个加密空间。
            </p>
          </div>
          <Button className="w-full" onClick={() => setShowMismatchHelp(false)} disabled={loading}>
            重新输入加密口令
          </Button>
          <Button variant="outline" className="w-full" onClick={onCreateNew} disabled={loading}>
            创建新的加密空间
          </Button>
        </div>
      ) : (
        <Button className="w-full" onClick={handleSubmit} disabled={loading}>
          {loading ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              验证中...
            </>
          ) : (
            '验证并继续'
          )}
        </Button>
      )}
    </motion.div>
  )
}
