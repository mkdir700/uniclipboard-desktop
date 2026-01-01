import { listen } from '@tauri-apps/api/event'
import { AnimatePresence, motion } from 'framer-motion'
import { CheckCircle, AlertCircle, Loader2, Key, Eye, EyeOff } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'
import { completeOnboarding, setupEncryptionPassword } from '@/api/onboarding'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useOnboarding } from '@/contexts/OnboardingContext'
import { cn } from '@/lib/utils'

type Step = 'welcome' | 'set-password' | 'confirming' | 'complete' | 'error'

interface OnboardingPasswordSetEvent {
  timestamp: number
}

interface OnboardingCompletedEvent {
  timestamp: number
}

// 进度条组件
const OnboardingProgress: React.FC<{ currentStep: number; totalSteps: number }> = ({
  currentStep,
  totalSteps,
}) => {
  return (
    <div className="fixed top-8 left-1/2 -translate-x-1/2 z-50">
      <div className="flex items-center gap-2">
        {Array.from({ length: totalSteps }).map((_, index) => (
          <div
            key={index}
            className={cn(
              'w-2 h-2 rounded-full transition-colors duration-300',
              currentStep >= index ? 'bg-foreground' : 'bg-foreground/20'
            )}
          />
        ))}
      </div>
    </div>
  )
}

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

  // 检查当前状态
  useEffect(() => {
    if (status === null) return

    // 如果已经完成了 onboarding，直接跳转
    if (status.has_completed) {
      navigate('/', { replace: true })
      return
    }
    // 如果密码已设置，跳过密码设置步骤
    if (status.encryption_password_set) {
      setStep('confirming')
    }
  }, [status, navigate])

  // 监听密码设置成功事件
  useEffect(() => {
    let unlisten: (() => void) | undefined

    const setupListener = async () => {
      unlisten = await listen<OnboardingPasswordSetEvent>('onboarding-password-set', () => {
        setStep('confirming')
        toast.success('密码设置成功')
      })
    }

    setupListener()
    return () => unlisten?.()
  }, [])

  // 监听完成事件，自动跳转
  useEffect(() => {
    let unlisten: (() => void) | undefined

    const setupListener = async () => {
      unlisten = await listen<OnboardingCompletedEvent>('onboarding-completed', () => {
        setStep('complete')
        toast.success('设置完成!')
        setLoading(false)
        // 短暂延迟后跳转，让用户看到成功状态
        setTimeout(() => navigate('/', { replace: true }), 2000)
      })
    }

    setupListener()
    return () => unlisten?.()
  }, [navigate])

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

  const handleComplete = async () => {
    setLoading(true)
    setError(null)

    // 设置 5 秒超时保护
    const timeoutId = setTimeout(async () => {
      const newStatus = await refreshStatus()
      if (newStatus.has_completed) {
        navigate('/', { replace: true })
      } else {
        setError('完成验证超时，请重试')
        setStep('error')
        setLoading(false)
      }
    }, 5000)

    try {
      await completeOnboarding()
      // 不再使用 setTimeout，等待事件触发跳转
    } catch (err) {
      clearTimeout(timeoutId)
      const errorMessage = err instanceof Error ? err.message : String(err)
      setError(errorMessage)
      setStep('error')
      setLoading(false)
      toast.error(errorMessage)
    }
  }

  // 计算当前步骤索引
  const getStepIndex = () => {
    switch (step) {
      case 'welcome':
        return 0
      case 'set-password':
        return 1
      case 'confirming':
      case 'complete':
        return 2
      default:
        return 0
    }
  }

  return (
    <div className="min-h-screen bg-background flex flex-col">
      {/* 进度条 - 仅在正常步骤显示 */}
      {step !== 'error' && <OnboardingProgress currentStep={getStepIndex()} totalSteps={3} />}

      {/* 内容区域 */}
      <div className="flex-1 flex flex-col items-center justify-center px-8 max-w-sm mx-auto">
        <AnimatePresence mode="wait" initial={false}>
          {/* 欢迎步骤 */}
          {step === 'welcome' && (
            <motion.div
              key="welcome"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -20 }}
              transition={{ duration: 0.4 }}
              className="w-full"
            >
              {/* Logo - 文字品牌 */}
              <motion.div
                initial={{ scale: 0.8, opacity: 0 }}
                animate={{ scale: 1, opacity: 1 }}
                transition={{ duration: 0.5 }}
                className="mb-8"
              >
                <h1 className="text-4xl font-bold bg-linear-to-r from-primary to-primary/70 bg-clip-text text-transparent text-center">
                  UniClipboard
                </h1>
              </motion.div>

              {/* 标题 */}
              <motion.h1
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.1 }}
                className="text-2xl font-bold text-foreground mb-2 text-center"
              >
                欢迎使用
              </motion.h1>

              {/* 描述 */}
              <motion.p
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.2 }}
                className="text-center text-muted-foreground mb-8 text-sm"
              >
                请设置一个加密密码来保护您的剪贴板数据
              </motion.p>

              {/* 使用说明卡片 */}
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.3 }}
                className="w-full mb-8 p-4 rounded-xl bg-muted/30 border border-border/40"
              >
                <div className="space-y-3">
                  <div className="flex items-start gap-2">
                    <div className="mt-0.5 w-4 h-4 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
                      <div className="w-1.5 h-1.5 rounded-full bg-primary" />
                    </div>
                    <span className="text-sm leading-relaxed text-foreground">
                      加密您的剪贴板数据
                    </span>
                  </div>
                  <div className="flex items-start gap-2">
                    <div className="mt-0.5 w-4 h-4 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
                      <div className="w-1.5 h-1.5 rounded-full bg-primary" />
                    </div>
                    <span className="text-sm leading-relaxed text-foreground">
                      在其他设备上解密同步的内容
                    </span>
                  </div>
                  <div className="flex items-start gap-2">
                    <div className="mt-0.5 w-4 h-4 rounded-full bg-destructive/10 flex items-center justify-center shrink-0">
                      <div className="w-1.5 h-1.5 rounded-full bg-destructive" />
                    </div>
                    <span className="text-sm leading-relaxed text-foreground">
                      丢失密码无法恢复数据
                    </span>
                  </div>
                </div>
              </motion.div>

              {/* 按钮 */}
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.4 }}
                className="w-full"
              >
                <Button
                  onClick={() => setStep('set-password')}
                  className="w-full h-11 text-sm font-medium rounded-xl shadow-lg shadow-primary/20"
                >
                  开始设置
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
              className="w-full"
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

          {/* 确认步骤 */}
          {step === 'confirming' && (
            <motion.div
              key="confirming"
              initial={{ scale: 0.9, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.9, opacity: 0 }}
              transition={{ duration: 0.3 }}
              className="w-full flex flex-col items-center"
            >
              {/* 图标 + Spinner 组合 */}
              <div className="relative mb-8">
                <motion.div
                  animate={{ rotate: 360 }}
                  transition={{ duration: 2, repeat: Infinity, ease: 'linear' }}
                  className="w-20 h-20"
                >
                  <div className="absolute inset-0 border-4 border-primary/20 rounded-full" />
                  <div className="absolute inset-0 border-4 border-primary border-t-transparent rounded-full" />
                </motion.div>
                <Key className="absolute inset-0 m-auto w-8 h-8 text-primary" />
              </div>

              {/* 标题 */}
              <h1 className="text-2xl font-bold text-center mb-2">正在设置您的设备</h1>

              {/* 描述 */}
              <p className="text-center text-muted-foreground text-sm mb-8">
                请稍候，我们正在初始化您的安全剪贴板同步
              </p>

              {/* 加载指示器 */}
              <div className="flex items-center gap-2 text-muted-foreground text-sm">
                <Loader2 className="w-4 h-4 animate-spin" />
                <span>初始化中...</span>
              </div>

              {/* 手动完成按钮（如果自动完成失败） */}
              {!loading && (
                <Button onClick={handleComplete} className="mt-8">
                  完成设置
                </Button>
              )}
            </motion.div>
          )}

          {/* 完成步骤 */}
          {step === 'complete' && (
            <motion.div
              key="complete"
              initial={{ scale: 0.8, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.8, opacity: 0 }}
              transition={{ type: 'spring', duration: 0.5 }}
              className="w-full flex flex-col items-center"
            >
              {/* 成功图标 */}
              <motion.div
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{ delay: 0.2, type: 'spring', stiffness: 200 }}
                className="mb-8"
              >
                <div className="w-20 h-20 rounded-full bg-green-100 dark:bg-green-900/20 flex items-center justify-center">
                  <CheckCircle className="w-10 h-10 text-green-600 dark:text-green-400" />
                </div>
              </motion.div>

              {/* 标题 */}
              <motion.h1
                initial={{ y: 20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.3 }}
                className="text-3xl font-bold text-center mb-3"
              >
                全部完成！
              </motion.h1>

              {/* 描述 */}
              <motion.p
                initial={{ y: 20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.4 }}
                className="text-center text-muted-foreground mb-12"
              >
                您的剪贴板同步已准备就绪
              </motion.p>

              {/* 配置摘要 */}
              <motion.div
                initial={{ y: 20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.5 }}
                className="w-full space-y-3 mb-12"
              >
                <div className="flex items-center gap-3 p-3 rounded-lg bg-muted/50">
                  <CheckCircle className="w-5 h-5 text-green-600 dark:text-green-400 shrink-0" />
                  <span className="text-sm">加密已配置</span>
                </div>
                <div className="flex items-center gap-3 p-3 rounded-lg bg-muted/50">
                  <CheckCircle className="w-5 h-5 text-green-600 dark:text-green-400 shrink-0" />
                  <span className="text-sm">设备已注册</span>
                </div>
              </motion.div>

              {/* 跳转提示 */}
              <motion.p
                initial={{ y: 20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.6 }}
                className="text-sm text-muted-foreground"
              >
                正在跳转到主页面...
              </motion.p>
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
              className="w-full flex flex-col items-center"
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
