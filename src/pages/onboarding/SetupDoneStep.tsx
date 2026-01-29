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

      <h1 className="text-2xl font-bold text-foreground mb-2">设置完成</h1>
      <p className="text-muted-foreground text-sm mb-8">您的设备已成功连接并加密，可以开始使用了</p>

      <div className="space-y-3 mb-8">
        {['剪贴板内容将自动加密同步', '支持文本、图片和文件传输', '随时在设置中管理已连接设备'].map(
          (text, i) => (
            <motion.div
              key={text}
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.2 + i * 0.1 }}
              className="flex items-center gap-3 p-3 rounded-xl bg-muted/30 border border-border/40 text-sm text-left"
            >
              <div className="w-1.5 h-1.5 rounded-full bg-green-500 shrink-0 ml-2" />
              <span className="text-foreground/80">{text}</span>
            </motion.div>
          )
        )}
      </div>

      <Button onClick={onComplete} className="w-full h-12 shadow-lg shadow-primary/20">
        开始使用
        <ArrowRight className="w-4 h-4 ml-2" />
      </Button>
    </motion.div>
  )
}
