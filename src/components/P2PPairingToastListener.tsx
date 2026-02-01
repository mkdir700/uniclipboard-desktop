import { useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { onP2PPairingVerification } from '@/api/p2p'
import { toast } from '@/components/ui/toast'

export default function P2PPairingToastListener() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const lastRequestSessionId = useRef<string | null>(null)

  useEffect(() => {
    const unlistenPromise = onP2PPairingVerification(event => {
      if (event.kind !== 'request') {
        return
      }

      if (event.sessionId && event.sessionId === lastRequestSessionId.current) {
        return
      }

      lastRequestSessionId.current = event.sessionId
      const deviceName = event.deviceName ?? t('pairing.discovery.unknownDevice')
      const description = t('pairing.globalRequest.description', { deviceName })
      const warning = t('pairing.globalRequest.warning')

      toast(t('pairing.globalRequest.title'), {
        description: `${description} ${warning}`,
        action: {
          label: t('pairing.globalRequest.action'),
          onClick: () => {
            navigate('/devices?pairing=1')
          },
        },
      })
    })

    return () => {
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [navigate, t])

  return null
}
