/**
 * 连接请求确认弹窗
 *
 * 当收到其他设备的连接请求时，显示此弹窗让用户确认
 */

import { Smartphone, Network, Loader2, CheckCircle2, XCircle } from 'lucide-react'
import React, { useState, useEffect, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import {
  respondToConnectionRequest,
  type ConnectionRequestInfo,
  type ConnectionRequestDecision,
} from '@/api/deviceConnection'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

interface ConnectionRequestModalProps {
  open: boolean
  onClose: () => void
  request: ConnectionRequestInfo | null
}

type RequestStatus = 'idle' | 'processing' | 'accepted' | 'rejected'

const ConnectionRequestModal: React.FC<ConnectionRequestModalProps> = ({
  open,
  onClose,
  request,
}) => {
  const { t } = useTranslation()
  const [status, setStatus] = useState<RequestStatus>('idle')
  const [timeLeft, setTimeLeft] = useState<number>(30)

  const handleResponse = useCallback(
    async (accept: boolean) => {
      if (!request) return

      setStatus('processing')

      try {
        const decision: ConnectionRequestDecision = {
          accept,
          requester_device_id: request.requester_device_id,
        }

        const response = await respondToConnectionRequest(decision)

        if (response.success) {
          setStatus(accept ? 'accepted' : 'rejected')

          // 延迟后关闭弹窗
          setTimeout(() => {
            onClose()
          }, 1500)
        } else {
          setStatus('idle')
          toast.error(
            response.message ||
              t('connectionRequest.status.error', {
                message: t('connectionRequest.status.processing'),
              })
          )
        }
      } catch (error) {
        console.error('Connection request error:', error)
        setStatus('idle')
        toast.error(
          t('connectionRequest.status.error', { message: t('connectionRequest.status.processing') })
        )
      }
    },
    [request, onClose, t]
  )

  // 倒计时自动拒绝
  useEffect(() => {
    if (!open || !request || status !== 'idle') return

    setTimeLeft(30)
    const timer = setInterval(() => {
      setTimeLeft(prev => {
        if (prev <= 1) {
          clearInterval(timer)
          // 自动拒绝
          handleResponse(false)
          return 0
        }
        return prev - 1
      })
    }, 1000)

    return () => clearInterval(timer)
  }, [open, request, status, handleResponse])

  // 重置状态
  useEffect(() => {
    if (open && request) {
      setStatus('idle')
      setTimeLeft(30)
    }
  }, [open, request])

  if (!request) return null

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t('connectionRequest.title')}</DialogTitle>
          <DialogDescription asChild>
            <div>
              {status === 'idle' && (
                <p className="text-muted-foreground">
                  {t('connectionRequest.status.idle', {
                    name: request.requester_alias || request.requester_device_id,
                  })}
                </p>
              )}
              {status === 'processing' && (
                <p className="text-muted-foreground">{t('connectionRequest.status.processing')}</p>
              )}
              {status === 'accepted' && (
                <p className="text-muted-foreground text-green-600">
                  {t('connectionRequest.status.accepted')}
                </p>
              )}
              {status === 'rejected' && (
                <p className="text-muted-foreground text-red-600">
                  {t('connectionRequest.status.rejected')}
                </p>
              )}
            </div>
          </DialogDescription>
        </DialogHeader>

        <div className="py-4">
          {status === 'idle' && (
            <div className="space-y-4">
              {/* 设备信息卡片 */}
              <div className="bg-muted rounded-lg p-5 border border-border/50">
                <div className="flex items-center gap-4">
                  <div className="p-3 bg-primary/10 rounded-md">
                    <Smartphone className="w-6 h-6 text-primary" />
                  </div>
                  <div className="flex-1">
                    <div className="font-semibold text-lg">
                      {request.requester_alias ||
                        t('connectionRequest.info.device', { id: request.requester_device_id })}
                    </div>
                    <div className="text-sm text-muted-foreground">
                      ID: {request.requester_device_id}
                    </div>
                  </div>
                </div>

                <div className="mt-4 pt-4 border-t border-border/50">
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Network className="w-4 h-4" />
                    <span>{t('connectionRequest.info.ip', { ip: request.requester_ip })}</span>
                  </div>
                  {request.requester_platform && (
                    <div className="text-xs text-muted-foreground mt-1">
                      {t('connectionRequest.info.platform', {
                        platform: request.requester_platform,
                      })}
                    </div>
                  )}
                </div>
              </div>

              {/* 倒计时提示 */}
              <div className="text-center text-sm text-muted-foreground">
                <span className={timeLeft <= 10 ? 'text-red-500' : ''}>
                  {t('connectionRequest.timeout', { time: timeLeft })}
                </span>
              </div>

              {/* 操作按钮 */}
              <div className="flex gap-3">
                <Button
                  onClick={() => handleResponse(false)}
                  variant="outline"
                  className="flex-1"
                  disabled={status !== 'idle'}
                >
                  {t('connectionRequest.actions.reject')}
                </Button>
                <Button
                  onClick={() => handleResponse(true)}
                  className="flex-1"
                  disabled={status !== 'idle'}
                >
                  {t('connectionRequest.actions.accept')}
                </Button>
              </div>
            </div>
          )}

          {status === 'processing' && (
            <div className="flex flex-col items-center py-6">
              <Loader2 className="w-12 h-12 animate-spin text-primary mb-4" />
              <p className="text-sm text-muted-foreground">
                {t('connectionRequest.status.establishing')}
              </p>
            </div>
          )}

          {status === 'accepted' && (
            <div className="flex flex-col items-center py-6">
              <CheckCircle2 className="w-12 h-12 text-green-500 mb-4" />
              <p className="text-sm font-medium text-green-600">
                {t('connectionRequest.status.connected')}
              </p>
            </div>
          )}

          {status === 'rejected' && (
            <div className="flex flex-col items-center py-6">
              <XCircle className="w-12 h-12 text-red-500 mb-4" />
              <p className="text-sm font-medium text-red-600">
                {t('connectionRequest.status.rejectedResult')}
              </p>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}

export default ConnectionRequestModal
