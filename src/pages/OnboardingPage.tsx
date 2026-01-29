import { AnimatePresence } from 'framer-motion'
import { useEffect, useState, useCallback } from 'react'
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

export default function OnboardingPage() {
  const navigate = useNavigate()
  const [setupState, setSetupState] = useState<SetupState | null>(null)
  const [loading, setLoading] = useState(false)
  const [peers, setPeers] = useState<Array<{ id: string; name: string; device_type: string }>>([])
  const [peersLoading, setPeersLoading] = useState(false)

  // Load initial state
  useEffect(() => {
    const loadState = async () => {
      try {
        const state = await getSetupState()
        setSetupState(state)
      } catch (error) {
        console.error('Failed to load setup state:', error)
        toast.error('加载设置状态失败')
      }
    }
    loadState()
  }, [])

  const handleRefreshPeers = useCallback(async () => {
    setPeersLoading(true)
    try {
      // Dispatch refresh event to backend to trigger discovery
      await dispatchSetupEvent('NetworkScanRefresh')

      // Then fetch the list
      const peerList = await getP2PPeers()
      setPeers(
        peerList.map((p: P2PPeerInfo) => ({
          id: p.peerId,
          name: p.deviceName || 'Unknown Device',
          device_type: 'desktop', // TODO: Get actual device type if available
        }))
      )
    } catch (error) {
      console.error('Failed to refresh peers:', error)
      toast.error('刷新设备列表失败')
    } finally {
      setPeersLoading(false)
    }
  }, [])

  // Fetch peers when entering JoinSpacePickDevice state
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
      toast.error('操作失败，请重试')
    } finally {
      setLoading(false)
    }
  }

  const renderStep = () => {
    if (!setupState) return null

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
            try {
              await completeOnboarding()
              navigate('/', { replace: true })
            } catch (error) {
              console.error('Failed to complete onboarding:', error)
              toast.error('完成设置失败')
            }
          }}
          loading={loading}
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

    return <div>Unknown state: {JSON.stringify(setupState)}</div>
  }

  return (
    <div className="h-full w-full bg-background flex flex-col">
      <div className="flex-1 flex flex-col items-center justify-center px-8 max-w-2xl mx-auto py-8 min-h-0 overflow-y-auto">
        <AnimatePresence mode="wait" initial={false}>
          {renderStep()}
        </AnimatePresence>
      </div>
    </div>
  )
}
