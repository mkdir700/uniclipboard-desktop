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
  error,
  loading,
}: JoinVerifyPassphraseStepProps) {
  const [passphrase, setPassphrase] = useState('')
  const [showPassphrase, setShowPassphrase] = useState(false)
  const [localError, setLocalError] = useState<string | null>(null)

  useEffect(() => {
    if (!error) {
      setLocalError(null)
      return
    }
    if (error === 'PassphraseInvalidOrMismatch') {
      setLocalError('密码错误，请重试')
    } else {
      setLocalError('验证失败，请重试')
    }
  }, [error])

  const handleSubmit = () => {
    if (!passphrase) {
      setLocalError('请输入密码')
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
          返回
        </Button>
        <h1 className="text-2xl font-bold text-foreground mt-2">输入密码</h1>
        <p className="text-muted-foreground text-sm mt-1">请输入目标设备的加密密码以进行配对</p>
        <p className="text-xs text-muted-foreground mt-1 font-mono">
          目标设备 ID: {peerId.substring(0, 8)}...
        </p>
      </div>

      <div className="space-y-5 mb-8">
        <div className="space-y-2">
          <Label htmlFor="passphrase">密码</Label>
          <div className="relative">
            <Input
              id="passphrase"
              type={showPassphrase ? 'text' : 'password'}
              value={passphrase}
              onChange={e => setPassphrase(e.target.value)}
              disabled={loading}
              className="pr-10"
              placeholder="输入密码"
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

      <Button className="w-full" onClick={handleSubmit} disabled={loading}>
        {loading ? (
          <>
            <Loader2 className="w-4 h-4 mr-2 animate-spin" />
            验证中...
          </>
        ) : (
          '验证并加入'
        )}
      </Button>
    </motion.div>
  )
}
