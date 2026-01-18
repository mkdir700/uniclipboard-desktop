import { listen } from '@tauri-apps/api/event'
import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'
import { checkOnboardingStatus, type OnboardingStatus } from '@/api/onboarding'

interface OnboardingContextType {
  status: OnboardingStatus | null
  loading: boolean
  error: string | null
  refreshStatus: () => Promise<OnboardingStatus>
}

const OnboardingContext = createContext<OnboardingContextType | undefined>(undefined)

export const useOnboarding = () => {
  const context = useContext(OnboardingContext)
  if (!context) {
    throw new Error('useOnboarding must be used within OnboardingProvider')
  }
  return context
}

interface OnboardingPasswordSetEvent {
  timestamp: number
}

interface OnboardingCompletedEvent {
  timestamp: number
}

export const OnboardingProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [status, setStatus] = useState<OnboardingStatus | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const refreshStatus = async () => {
    try {
      setLoading(true)
      const newStatus = await checkOnboardingStatus()
      setStatus(newStatus)
      setError(null)
      return newStatus
    } catch (err) {
      setError(String(err))
      console.error('Failed to refresh onboarding status:', err)
      throw err
    } finally {
      setLoading(false)
    }
  }

  // Wait for backend-ready event before checking onboarding status
  // This prevents blocking the initial render with an IPC call
  useEffect(() => {
    const unlistenPromise = listen('backend-ready', async () => {
      // Now that backend is ready, check the onboarding status
      await refreshStatus()
    })

    return () => {
      unlistenPromise.then(unlisten => unlisten?.()).catch(() => {})
    }
  }, [])

  // 监听后端事件
  useEffect(() => {
    let unlistenPasswordSet: (() => void) | undefined
    let unlistenCompleted: (() => void) | undefined

    const setupListeners = async () => {
      try {
        unlistenPasswordSet = await listen<OnboardingPasswordSetEvent>(
          'onboarding-password-set',
          () => {
            console.log('Password set event received')
            refreshStatus()
          }
        )

        unlistenCompleted = await listen<OnboardingCompletedEvent>('onboarding-completed', () => {
          console.log('Onboarding completed event received')
          refreshStatus()
        })
      } catch (err) {
        console.error('Failed to setup onboarding listeners:', err)
      }
    }

    setupListeners()

    return () => {
      unlistenPasswordSet?.()
      unlistenCompleted?.()
    }
  }, [])

  return (
    <OnboardingContext.Provider value={{ status, loading, error, refreshStatus }}>
      {children}
    </OnboardingContext.Provider>
  )
}
