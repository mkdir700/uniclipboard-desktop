import { motion } from 'framer-motion'
import { AlertCircle, Eye, EyeOff, Loader2, ArrowLeft } from 'lucide-react'
import { useState, useEffect } from 'react'
import { CreatePassphraseStepProps } from './types'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

export default function CreatePassphraseStep({
  onSubmit,
  onBack,
  error,
  loading,
}: CreatePassphraseStepProps) {
  const [pass1, setPass1] = useState('')
  const [pass2, setPass2] = useState('')
  const [showPass1, setShowPass1] = useState(false)
  const [showPass2, setShowPass2] = useState(false)
  const [localError, setLocalError] = useState<string | null>(null)

  // Map SetupError to user-friendly message
  useEffect(() => {
    if (!error) {
      setLocalError(null)
      return
    }

    if (error === 'PassphraseMismatch') {
      setLocalError('两次输入不一致，请重新确认。')
    } else if (typeof error === 'object' && 'PassphraseTooShort' in error) {
      setLocalError(`口令太短，请至少输入 ${error.PassphraseTooShort.min_len} 位。`)
    } else if (error === 'PassphraseEmpty') {
      setLocalError('请输入加密口令。')
    } else {
      setLocalError('设置失败，请重试')
    }
  }, [error])

  const handleSubmit = () => {
    if (!pass1) {
      setLocalError('请输入加密口令。')
      return
    }
    if (pass1 !== pass2) {
      setLocalError('两次输入不一致，请重新确认。')
      return
    }
    onSubmit(pass1, pass2)
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
        <h1 className="text-2xl font-bold text-foreground mt-2">设置加密口令</h1>
        <p className="text-muted-foreground text-sm mt-1">这个口令将创建你的加密空间。</p>
        <p className="text-xs text-muted-foreground mt-3">
          之后想在其他设备上使用 UniClipboard，需要输入相同的口令才能加入这个空间并共享剪贴板。
        </p>
      </div>

      <div className="space-y-5 mb-8">
        <div className="space-y-2">
          <Label htmlFor="pass1">加密口令</Label>
          <div className="relative">
            <Input
              id="pass1"
              type={showPass1 ? 'text' : 'password'}
              value={pass1}
              onChange={e => setPass1(e.target.value)}
              disabled={loading}
              className="pr-10"
              placeholder="输入加密口令"
            />
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
              onClick={() => setShowPass1(!showPass1)}
            >
              {showPass1 ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
            </Button>
          </div>
        </div>

        <div className="space-y-2">
          <Label htmlFor="pass2">确认口令</Label>
          <div className="relative">
            <Input
              id="pass2"
              type={showPass2 ? 'text' : 'password'}
              value={pass2}
              onChange={e => setPass2(e.target.value)}
              disabled={loading}
              className="pr-10"
              placeholder="再次输入"
              onKeyDown={e => e.key === 'Enter' && handleSubmit()}
            />
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
              onClick={() => setShowPass2(!showPass2)}
            >
              {showPass2 ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
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

      <p className="text-xs text-muted-foreground mb-6">
        建议选择一个你愿意在所有设备上使用、并且能记住的口令。
      </p>

      <Button className="w-full" onClick={handleSubmit} disabled={loading}>
        {loading ? (
          <>
            <Loader2 className="w-4 h-4 mr-2 animate-spin" />
            设置中...
          </>
        ) : (
          '创建并进入'
        )}
      </Button>
    </motion.div>
  )
}
