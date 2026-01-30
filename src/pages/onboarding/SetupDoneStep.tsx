import { motion } from 'framer-motion'
import { CheckCircle2, ArrowRight } from 'lucide-react'
import { SetupDoneStepProps } from './types'
import { Button } from '@/components/ui/button'

export default function SetupDoneStep({ onComplete }: SetupDoneStepProps) {
  return (
    <motion.div
      initial={{ scale: 0.9, opacity: 0 }}
      animate={{ scale: 1, opacity: 1 }}
      exit={{ scale: 0.9, opacity: 0 }}
      className="w-full max-w-md mx-auto text-center"
    >
      <div className="mb-8 flex justify-center">
        <div className="w-20 h-20 rounded-full bg-green-500/10 flex items-center justify-center">
          <CheckCircle2 className="w-10 h-10 text-green-500" />
        </div>
      </div>

      <h1 className="text-2xl font-bold text-foreground mb-2">已完成设置</h1>
      <p className="text-muted-foreground text-sm mb-8">
        你的设备已准备就绪，现在可以安全共享剪贴板了。
      </p>

      <Button onClick={onComplete} className="w-full h-12 shadow-lg shadow-primary/20">
        进入主页
        <ArrowRight className="w-4 h-4 ml-2" />
      </Button>
    </motion.div>
  )
}
