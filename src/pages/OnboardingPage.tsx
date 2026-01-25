import { AnimatePresence, motion } from 'framer-motion'
import { AlertCircle, Loader2, Eye, EyeOff } from 'lucide-react'
import { useEffect, useState, useCallback, useRef } from 'react'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'
import { completeOnboarding, setupEncryptionPassword } from '@/api/onboarding'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useOnboarding } from '@/contexts/OnboardingContext'
import { cn } from '@/lib/utils'

type Step = 'welcome' | 'set-password' | 'error'

export default function OnboardingPage() {
  const navigate = useNavigate()
  const { status, refreshStatus } = useOnboarding()
  const [step, setStep] = useState<Step>('welcome')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // 密码相关状态
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [showConfirmPassword, setShowConfirmPassword] = useState(false)
  const [passwordError, setPasswordError] = useState<string | null>(null)
  const completionRequestedRef = useRef(false)
  const completionTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const clearCompletionTimeout = useCallback(() => {
    if (completionTimeoutRef.current) {
      clearTimeout(completionTimeoutRef.current)
      completionTimeoutRef.current = null
    }
  }, [])

  const handleComplete = useCallback(async () => {
    if (completionRequestedRef.current) {
      return
    }
    completionRequestedRef.current = true
    setLoading(true)
    setError(null)

    // 设置 5 秒超时保护
    clearCompletionTimeout()
    completionTimeoutRef.current = setTimeout(async () => {
      try {
        const newStatus = await refreshStatus()
        if (newStatus.has_completed) {
          clearCompletionTimeout()
          navigate('/', { replace: true })
        } else {
          completionRequestedRef.current = false
          setError('完成验证超时，请重试')
          setStep('error')
          setLoading(false)
        }
      } catch (err) {
        clearCompletionTimeout()
        completionRequestedRef.current = false
        const errorMessage = err instanceof Error ? err.message : String(err)
        setError(errorMessage)
        setStep('error')
        setLoading(false)
        toast.error(errorMessage)
      }
    }, 5000)

    try {
      await completeOnboarding()
      // 不再使用 setTimeout，等待事件触发跳转
    } catch (err) {
      clearCompletionTimeout()
      completionRequestedRef.current = false
      const errorMessage = err instanceof Error ? err.message : String(err)
      setError(errorMessage)
      setStep('error')
      setLoading(false)
      toast.error(errorMessage)
    }
  }, [clearCompletionTimeout, navigate, refreshStatus])

  // 检查当前状态
  useEffect(() => {
    if (status === null) return

    // 如果已经完成了 onboarding，直接跳转
    if (status.has_completed) {
      clearCompletionTimeout()
      completionRequestedRef.current = false
      navigate('/', { replace: true })
      return
    }
    // 如果密码已设置，直接尝试完成
    if (status.encryption_password_set) {
      handleComplete()
    }
  }, [clearCompletionTimeout, status, navigate, handleComplete])

  useEffect(() => () => clearCompletionTimeout(), [clearCompletionTimeout])

  const validatePassword = (): boolean => {
    if (password.length < 8) {
      setPasswordError('密码长度至少为 8 位')
      return false
    }
    if (password !== confirmPassword) {
      setPasswordError('两次输入的密码不一致')
      return false
    }
    setPasswordError(null)
    return true
  }

  const handleSetPassword = async () => {
    if (!validatePassword()) {
      return
    }

    setLoading(true)
    setError(null)

    try {
      await setupEncryptionPassword(password)
      // 不再手动设置 step，等待事件触发
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      setError(errorMessage)
      toast.error(errorMessage)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="h-full w-full bg-background flex flex-col">
      {/* 内容区域 */}
      <div className="flex-1 flex flex-col items-center justify-center px-8 max-w-2xl mx-auto py-8 min-h-0 overflow-y-auto">
        <AnimatePresence mode="wait" initial={false}>
          {/* 欢迎步骤 */}
          {step === 'welcome' && (
            <motion.div
              key="welcome"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -20 }}
              transition={{ duration: 0.4 }}
              className="w-full max-w-2xl mx-auto"
            >
              <div className="text-center mb-6">
                <motion.h1
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 0.1 }}
                  className="text-3xl font-bold text-foreground mb-2 tracking-tight"
                >
                  欢迎来到 UniClipboard
                </motion.h1>
                <motion.h2
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 0.2 }}
                  className="text-xl font-semibold text-foreground/80 mb-3"
                >
                  现在设置加密密码
                </motion.h2>
                <motion.p
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 0.3 }}
                  className="text-muted-foreground text-sm"
                >
                  这是首次启动的安全设置，用于保护同步内容
                </motion.p>
              </div>

              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.4 }}
                className="grid gap-3 mb-6"
              >
                {[
                  '创建密码后，本机剪贴板会被加密保存',
                  '在其他设备上使用同一密码解锁同步内容',
                  '设置完成即可开始安全同步',
                ].map(text => (
                  <div
                    key={text}
                    className="flex items-center gap-3 p-4 rounded-xl bg-muted/30 border border-border/40"
                  >
                    <div className="w-2 h-2 rounded-full bg-primary/60 shrink-0" />
                    <span className="text-sm text-foreground font-medium">{text}</span>
                  </div>
                ))}
              </motion.div>

              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.5 }}
                className="flex items-center gap-2 justify-center mb-6 text-destructive text-xs font-medium bg-destructive/5 py-1.5 px-3 rounded-full w-fit mx-auto"
              >
                <AlertCircle className="w-4 h-4" />
                <span>丢失密码将无法恢复已加密数据</span>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.6 }}
                className="flex justify-center"
              >
                <Button
                  onClick={() => setStep('set-password')}
                  className="w-full max-w-sm h-11 text-sm font-medium rounded-xl shadow-lg shadow-primary/20"
                >
                  开始设置密码
                </Button>
              </motion.div>
            </motion.div>
          )}

          {/* 设置密码步骤 */}
          {step === 'set-password' && (
            <motion.div
              key="set-password"
              initial={{ x: 300, opacity: 0 }}
              animate={{ x: 0, opacity: 1 }}
              exit={{ x: -300, opacity: 0 }}
              transition={{ type: 'spring', damping: 25, stiffness: 200 }}
              className="w-full max-w-2xl mx-auto"
            >
              {/* 标题 */}
              <h1 className="text-2xl font-bold text-foreground mb-2">创建您的密码</h1>

              {/* 描述 */}
              <p className="text-muted-foreground mb-8 text-sm">设置一个密码来加密您的剪贴板数据</p>

              {/* 表单 */}
              <div className="space-y-5 mb-8">
                <div className="space-y-2">
                  <Label htmlFor="password" className="text-sm font-medium text-foreground">
                    密码
                  </Label>
                  <div className="relative">
                    <Input
                      id="password"
                      type={showPassword ? 'text' : 'password'}
                      placeholder="至少 8 位字符"
                      value={password}
                      onChange={e => setPassword(e.target.value)}
                      onKeyDown={e => e.key === 'Enter' && handleSetPassword()}
                      disabled={loading}
                      className={cn(
                        'h-12 pr-12 text-sm rounded-xl',
                        passwordError && 'border-destructive focus-visible:ring-destructive',
                        'bg-muted/50 focus-visible:bg-background'
                      )}
                    />
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="absolute right-1 top-1/2 -translate-y-1/2 h-9 w-9 p-0 rounded-lg"
                      onClick={() => setShowPassword(!showPassword)}
                    >
                      {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                    </Button>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="confirm-password" className="text-sm font-medium text-foreground">
                    确认密码
                  </Label>
                  <div className="relative">
                    <Input
                      id="confirm-password"
                      type={showConfirmPassword ? 'text' : 'password'}
                      placeholder="再次输入密码"
                      value={confirmPassword}
                      onChange={e => setConfirmPassword(e.target.value)}
                      onKeyDown={e => e.key === 'Enter' && handleSetPassword()}
                      disabled={loading}
                      className={cn(
                        'h-12 pr-12 text-sm rounded-xl',
                        passwordError && 'border-destructive focus-visible:ring-destructive',
                        'bg-muted/50 focus-visible:bg-background'
                      )}
                    />
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="absolute right-1 top-1/2 -translate-y-1/2 h-9 w-9 p-0 rounded-lg"
                      onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                    >
                      {showConfirmPassword ? (
                        <EyeOff className="h-4 w-4" />
                      ) : (
                        <Eye className="h-4 w-4" />
                      )}
                    </Button>
                  </div>
                </div>

                {passwordError && (
                  <motion.div
                    initial={{ opacity: 0, y: -10 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="p-3 rounded-xl bg-destructive/10 border border-destructive/20"
                  >
                    <p className="text-sm text-destructive flex items-center gap-2">
                      <AlertCircle className="h-4 w-4 shrink-0" />
                      {passwordError}
                    </p>
                  </motion.div>
                )}
              </div>

              {/* 按钮组 */}
              <div className="flex gap-3">
                <Button
                  variant="outline"
                  onClick={() => setStep('welcome')}
                  disabled={loading}
                  className="flex-1 h-11 text-sm rounded-xl"
                >
                  返回
                </Button>
                <Button
                  onClick={handleSetPassword}
                  disabled={loading}
                  className="flex-1 h-11 text-sm rounded-xl shadow-lg shadow-primary/20"
                >
                  {loading ? (
                    <>
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                      设置中...
                    </>
                  ) : (
                    '继续'
                  )}
                </Button>
              </div>
            </motion.div>
          )}

          {/* 错误步骤 */}
          {step === 'error' && (
            <motion.div
              key="error"
              initial={{ scale: 0.9, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.9, opacity: 0 }}
              transition={{ duration: 0.3 }}
              className="w-full max-w-sm mx-auto flex flex-col items-center"
            >
              {/* 错误图标 */}
              <div className="mb-8">
                <div className="w-16 h-16 rounded-2xl bg-destructive/10 flex items-center justify-center">
                  <AlertCircle className="w-8 h-8 text-destructive" />
                </div>
              </div>

              {/* 标题 */}
              <h1 className="text-2xl font-bold text-center mb-2">出现错误</h1>

              {/* 描述 */}
              <p className="text-center text-muted-foreground text-sm mb-8 max-w-sm">
                {error || '设置失败，请重试'}
              </p>

              {/* 按钮 */}
              <Button
                variant="outline"
                onClick={() => {
                  setError(null)
                  setStep('welcome')
                }}
                className="w-full h-11"
              >
                返回
              </Button>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  )
}
