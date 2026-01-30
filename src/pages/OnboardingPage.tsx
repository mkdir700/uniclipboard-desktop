import { AnimatePresence } from 'framer-motion'
import { Loader2, Shield, Wifi, Key } from 'lucide-react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'
import CreatePassphraseStep from './onboarding/CreatePassphraseStep'
import JoinPickDeviceStep from './onboarding/JoinPickDeviceStep'
import JoinVerifyPassphraseStep from './onboarding/JoinVerifyPassphraseStep'
import PairingConfirmStep from './onboarding/PairingConfirmStep'
import SetupDoneStep from './onboarding/SetupDoneStep'
import WelcomeStep from './onboarding/WelcomeStep'
import {
  getSetupState,
  dispatchSetupEvent,
  completeOnboarding,
  SetupState,
  SetupEvent,
} from '@/api/onboarding'
import { getP2PPeers, P2PPeerInfo } from '@/api/p2p'
import { useOnboarding } from '@/contexts/onboarding-context'

export default function OnboardingPage() {
  const { t } = useTranslation(undefined, { keyPrefix: 'onboarding.page' })
  const { t: tCommon } = useTranslation(undefined, { keyPrefix: 'onboarding.common' })
  const navigate = useNavigate()
  const { refreshStatus } = useOnboarding()
  const [setupState, setSetupState] = useState<SetupState | null>(null)
  const [loading, setLoading] = useState(false)
  const [completing, setCompleting] = useState(false)
  const [peers, setPeers] = useState<Array<{ id: string; name: string; device_type: string }>>([])
  const [peersLoading, setPeersLoading] = useState(false)

  useEffect(() => {
    const loadState = async () => {
      try {
        const state = await getSetupState()
        setSetupState(state)
      } catch (error) {
        console.error('Failed to load setup state:', error)
        toast.error(t('errors.loadSetupStateFailed'))
      }
    }
    loadState()
  }, [t])

  const handleRefreshPeers = useCallback(async () => {
    setPeersLoading(true)
    try {
      await dispatchSetupEvent('NetworkScanRefresh')
      const peerList = await getP2PPeers()
      setPeers(
        peerList.map((p: P2PPeerInfo) => ({
          id: p.peerId,
          name: p.deviceName || tCommon('unknownDevice'),
          device_type: 'desktop',
        }))
      )
    } catch (error) {
      console.error('Failed to refresh peers:', error)
      toast.error(t('errors.refreshPeersFailed'))
    } finally {
      setPeersLoading(false)
    }
  }, [t, tCommon])

  useEffect(() => {
    if (setupState && typeof setupState === 'object' && 'JoinSpacePickDevice' in setupState) {
      handleRefreshPeers()
    }
  }, [setupState, handleRefreshPeers])

  const handleDispatch = async (event: SetupEvent) => {
    setLoading(true)
    try {
      const newState = await dispatchSetupEvent(event)
      setSetupState(newState)
    } catch (error) {
      console.error('Failed to dispatch event:', error)
      toast.error(t('errors.operationFailed'))
    } finally {
      setLoading(false)
    }
  }

  const renderStep = () => {
    if (!setupState) {
      return (
        <div className="flex h-full w-full items-center justify-center">
          <div className="flex items-center gap-3 text-sm text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" />
            {t('loadingSetupState')}
          </div>
        </div>
      )
    }

    if (setupState === 'Welcome') {
      return (
        <WelcomeStep
          onCreate={() => handleDispatch('ChooseCreateSpace')}
          onJoin={() => handleDispatch('ChooseJoinSpace')}
          loading={loading}
        />
      )
    }

    if (setupState === 'Done') {
      return (
        <SetupDoneStep
          onComplete={async () => {
            setCompleting(true)
            try {
              await completeOnboarding()
              await refreshStatus()
              navigate('/', { replace: true })
            } catch (error) {
              console.error('Failed to complete onboarding:', error)
              toast.error(t('errors.completeSetupFailed'))
            } finally {
              setCompleting(false)
            }
          }}
          loading={completing}
        />
      )
    }

    if (typeof setupState === 'object') {
      if ('CreateSpacePassphrase' in setupState) {
        return (
          <CreatePassphraseStep
            onSubmit={(pass1, pass2) =>
              handleDispatch({ SubmitCreatePassphrase: { pass1, pass2 } })
            }
            onBack={() => handleDispatch('Back')}
            error={setupState.CreateSpacePassphrase.error}
            loading={loading}
          />
        )
      }

      if ('JoinSpacePickDevice' in setupState) {
        return (
          <JoinPickDeviceStep
            onSelectPeer={peerId => handleDispatch({ SelectPeer: { peer_id: peerId } })}
            onBack={() => handleDispatch('Back')}
            onRefresh={handleRefreshPeers}
            peers={peers}
            error={setupState.JoinSpacePickDevice.error}
            loading={loading || peersLoading}
          />
        )
      }

      if ('JoinSpaceVerifyPassphrase' in setupState) {
        const { peer_id, error } = setupState.JoinSpaceVerifyPassphrase
        return (
          <JoinVerifyPassphraseStep
            peerId={peer_id}
            onSubmit={passphrase => handleDispatch({ SubmitJoinPassphrase: { passphrase } })}
            onBack={() => handleDispatch('Back')}
            onCreateNew={() => handleDispatch('ChooseCreateSpace')}
            error={error}
            loading={loading}
          />
        )
      }

      if ('PairingConfirm' in setupState) {
        const { short_code, session_id, peer_fingerprint, error } = setupState.PairingConfirm
        return (
          <PairingConfirmStep
            shortCode={short_code}
            sessionId={session_id}
            peerFingerprint={peer_fingerprint}
            onConfirm={() => handleDispatch('PairingUserConfirm')}
            onCancel={() => handleDispatch('PairingUserCancel')}
            error={error}
            loading={loading}
          />
        )
      }
    }

    return <div>{t('unknownState', { state: JSON.stringify(setupState) })}</div>
  }

  const stepKey = useMemo(() => {
    if (!setupState) return 'loading'
    if (typeof setupState === 'string') return setupState
    return Object.keys(setupState)[0] ?? 'unknown'
  }, [setupState])

  return (
    <div className="relative h-full w-full overflow-hidden bg-background">
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute inset-0 bg-gradient-to-br from-background via-background to-muted/20" />
        <div className="absolute -top-32 -left-32 h-96 w-96 bg-primary/5 blur-3xl" />
        <div className="absolute -bottom-32 -right-32 h-96 w-96 bg-emerald-500/5 blur-3xl" />
      </div>

      <div className="relative flex h-full w-full flex-col">
        <main className="flex flex-1 items-center overflow-y-auto px-6 py-12 lg:px-16">
          <div className="mx-auto w-full max-w-2xl">
            <AnimatePresence mode="wait" initial={false}>
              <div key={stepKey}>{renderStep()}</div>
            </AnimatePresence>
          </div>
        </main>

        <div className="pointer-events-none absolute bottom-6 right-6 hidden flex-col gap-2 text-[10px] text-muted-foreground/60 lg:flex">
          <div className="flex items-center gap-1.5">
            <Shield className="h-3 w-3" />
            <span>{t('badges.e2ee')}</span>
          </div>
          <div className="flex items-center gap-1.5">
            <Key className="h-3 w-3" />
            <span>{t('badges.localKeys')}</span>
          </div>
          <div className="flex items-center gap-1.5">
            <Wifi className="h-3 w-3" />
            <span>{t('badges.lanDiscovery')}</span>
          </div>
        </div>
      </div>
    </div>
  )
}
