import { motion } from 'framer-motion'
import { Shield, Smartphone } from 'lucide-react'
import { WelcomeStepProps } from './types'
import { Button } from '@/components/ui/button'

export default function WelcomeStep({ onCreate, onJoin, loading }: WelcomeStepProps) {
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
      <p className="text-base text-foreground/80 mb-3">你想如何开始？</p>
      <p className="text-sm text-muted-foreground mb-8 max-w-xl mx-auto">
        UniClipboard
        使用端到端加密。你可以创建一个新的加密空间，或加入已存在的空间，与其他设备安全共享剪贴板。
      </p>

      <div className="grid gap-4 md:grid-cols-2 max-w-xl mx-auto">
        <Button
          variant="outline"
          className="h-auto p-6 flex flex-col items-center gap-4 hover:bg-primary/5 hover:border-primary/30 transition-all group"
          onClick={onCreate}
          disabled={loading}
        >
          <div className="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center group-hover:bg-primary/20 transition-colors">
            <Shield className="w-6 h-6 text-primary" />
          </div>
          <div className="space-y-1">
            <h3 className="font-semibold text-foreground">创建新的加密空间</h3>
            <p className="text-xs text-muted-foreground">
              适用于这是你的第一台设备，或你想从一个全新的空间开始。
            </p>
          </div>
        </Button>

        <Button
          variant="outline"
          className="h-auto p-6 flex flex-col items-center gap-4 hover:bg-primary/5 hover:border-primary/30 transition-all group"
          onClick={onJoin}
          disabled={loading}
        >
          <div className="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center group-hover:bg-primary/20 transition-colors">
            <Smartphone className="w-6 h-6 text-primary" />
          </div>
          <div className="space-y-1">
            <h3 className="font-semibold text-foreground">加入已有的加密空间</h3>
            <p className="text-xs text-muted-foreground">
              如果你已经在另一台设备上使用 UniClipboard，选择此项即可加入并同步。
            </p>
          </div>
        </Button>
      </div>
    </motion.div>
  )
}
