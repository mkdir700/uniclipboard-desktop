import { motion } from 'framer-motion'
import { ArrowLeft, RefreshCw, Monitor, Smartphone, Laptop, AlertCircle } from 'lucide-react'
import { JoinPickDeviceStepProps } from './types'
import { Button } from '@/components/ui/button'

export default function JoinPickDeviceStep({
  onSelectPeer,
  onBack,
  onRefresh,
  peers,
  error,
  loading,
}: JoinPickDeviceStepProps) {
  const getIcon = (type: string) => {
    switch (type.toLowerCase()) {
      case 'mobile':
        return <Smartphone className="w-5 h-5" />
      case 'desktop':
        return <Monitor className="w-5 h-5" />
      default:
        return <Laptop className="w-5 h-5" />
    }
  }

  return (
    <motion.div
      initial={{ x: 300, opacity: 0 }}
      animate={{ x: 0, opacity: 1 }}
      exit={{ x: -300, opacity: 0 }}
      className="w-full max-w-md mx-auto"
    >
      <div className="mb-6">
        <Button
          variant="ghost"
          size="sm"
          className="pl-0 hover:bg-transparent text-muted-foreground hover:text-foreground"
          onClick={onBack}
        >
          <ArrowLeft className="w-4 h-4 mr-1" />
          返回
        </Button>
        <div className="flex items-center justify-between mt-2">
          <h1 className="text-2xl font-bold text-foreground">加入已有的加密空间</h1>
          <Button
            variant="ghost"
            size="icon"
            onClick={onRefresh}
            disabled={loading}
            className={loading ? 'animate-spin' : ''}
          >
            <RefreshCw className="w-4 h-4" />
          </Button>
        </div>
        <p className="text-muted-foreground text-sm mt-1">
          请选择一台已经在使用 UniClipboard 的设备。
        </p>
      </div>

      {error && (
        <div className="mb-6 p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-sm flex items-center gap-2">
          <AlertCircle className="w-4 h-4 shrink-0" />
          {error === 'NetworkTimeout' ? '扫描超时，请重试。' : '获取设备列表失败'}
        </div>
      )}

      <div className="space-y-3">
        {peers.length === 0 ? (
          <div className="text-center py-12 text-muted-foreground bg-muted/30 rounded-xl border border-border/50">
            <p className="text-base text-foreground">未发现可加入的设备</p>
            <p className="text-xs mt-2 opacity-70">
              请确认两台设备在同一网络下，并且另一台设备已打开 UniClipboard。
            </p>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={onRefresh}
              disabled={loading}
              className="mt-4 gap-2"
            >
              <RefreshCw className={`w-3 h-3 ${loading ? 'animate-spin' : ''}`} />
              重新扫描
            </Button>
          </div>
        ) : (
          peers.map(peer => (
            <div
              key={peer.id}
              className="w-full flex items-center gap-4 p-4 rounded-xl border border-border/50 bg-card"
            >
              <div className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center text-primary shrink-0">
                {getIcon(peer.device_type)}
              </div>
              <div className="flex-1 min-w-0">
                <div className="font-medium truncate">{peer.name}</div>
                <div className="text-xs text-muted-foreground truncate font-mono opacity-70">
                  {peer.id.substring(0, 8)}...
                </div>
              </div>
              <Button
                type="button"
                size="sm"
                onClick={() => onSelectPeer(peer.id)}
                disabled={loading}
              >
                继续
              </Button>
            </div>
          ))
        )}
      </div>
    </motion.div>
  )
}
