import { Bell, ShieldCheck } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { P2PPairingRequestEvent } from '@/api/p2p'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

interface GlobalPairingRequestDialogProps {
  open: boolean
  request: P2PPairingRequestEvent | null
  onAccept: () => Promise<void>
  onReject: () => Promise<void>
}

export default function GlobalPairingRequestDialog({
  open,
  request,
  onAccept,
  onReject,
}: GlobalPairingRequestDialogProps) {
  const { t } = useTranslation()
  const [accepting, setAccepting] = useState(false)

  const handleAccept = async () => {
    if (!request) return

    setAccepting(true)
    try {
      await onAccept()
    } catch (error) {
      console.error('Failed to accept pairing:', error)
    } finally {
      setAccepting(false)
    }
  }

  const handleReject = async () => {
    if (!request) return

    try {
      await onReject()
    } catch (error) {
      console.error('Failed to reject pairing:', error)
    }
  }

  if (!request) return null

  return (
    <Dialog open={open} onOpenChange={open => !open && handleReject()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary/10 rounded-full">
              <Bell className="w-5 h-5 text-primary" />
            </div>
            <DialogTitle>{t('pairing.globalRequest.title')}</DialogTitle>
          </div>
          <DialogDescription className="pt-2">
            {t('pairing.globalRequest.description', {
              deviceName: request.deviceName || t('pairing.discovery.unknownDevice'),
            })}
          </DialogDescription>
        </DialogHeader>

        <div className="py-4">
          <div className="flex items-center gap-2 text-sm text-muted-foreground bg-amber-500/10 text-amber-600 px-4 py-2 rounded-full">
            <ShieldCheck className="w-4 h-4" />
            {t('pairing.globalRequest.warning')}
          </div>
        </div>

        <DialogFooter className="gap-2">
          <Button variant="outline" onClick={handleReject} disabled={accepting}>
            {t('pairing.requests.reject')}
          </Button>
          <Button onClick={handleAccept} disabled={accepting}>
            {accepting ? t('pairing.requests.accepting') : t('pairing.requests.accept')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
