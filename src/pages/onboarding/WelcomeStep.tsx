import { motion } from 'framer-motion'
import { Shield, Smartphone } from 'lucide-react'
import { WelcomeStepProps } from './types'
import { Button } from '@/components/ui/button'

export default function WelcomeStep({ onCreate, onJoin }: WelcomeStepProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      className="w-full max-w-2xl mx-auto text-center"
    >
      <h1 className="text-3xl font-bold text-foreground mb-2 tracking-tight">
        欢迎使用 UniClipboard
      </h1>
      <p className="text-muted-foreground mb-8">安全、跨平台的剪贴板同步工具</p>

      <div className="grid gap-4 md:grid-cols-2 max-w-xl mx-auto">
        <Button
          variant="outline"
          className="h-auto p-6 flex flex-col items-center gap-4 hover:bg-primary/5 hover:border-primary/30 transition-all group"
          onClick={onCreate}
        >
          <div className="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center group-hover:bg-primary/20 transition-colors">
            <Shield className="w-6 h-6 text-primary" />
          </div>
          <div className="space-y-1">
            <h3 className="font-semibold text-foreground">创建新的加密空间</h3>
            <p className="text-xs text-muted-foreground">我是第一台设备，需要设置新密码</p>
          </div>
        </Button>

        <Button
          variant="outline"
          className="h-auto p-6 flex flex-col items-center gap-4 hover:bg-primary/5 hover:border-primary/30 transition-all group"
          onClick={onJoin}
        >
          <div className="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center group-hover:bg-primary/20 transition-colors">
            <Smartphone className="w-6 h-6 text-primary" />
          </div>
          <div className="space-y-1">
            <h3 className="font-semibold text-foreground">加入已有加密空间</h3>
            <p className="text-xs text-muted-foreground">已有其他设备，通过配对加入</p>
          </div>
        </Button>
      </div>
    </motion.div>
  )
}
