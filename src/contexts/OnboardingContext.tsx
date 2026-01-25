import { listen } from '@tauri-apps/api/event'
import { createContext, useCallback, useContext, useEffect, type ReactNode } from 'react'
import { type OnboardingStatus } from '@/api/onboarding'
import { appApi, useGetOnboardingStatusQuery } from '@/store/api'
import { useAppDispatch } from '@/store/hooks'

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
  const dispatch = useAppDispatch()
  const { data, isLoading, error, refetch } = useGetOnboardingStatusQuery()

  const refreshStatus = useCallback(async () => {
    const result = await refetch()
    if ('error' in result) {
      const errorMessage =
        result.error && typeof result.error === 'object' && 'message' in result.error
          ? String(result.error.message)
          : 'Failed to refresh onboarding status'
      throw new Error(errorMessage)
    }

    if (!result.data) {
      throw new Error('Missing onboarding status')
    }

    return result.data
  }, [refetch])

  const errorMessage =
    error && typeof error === 'object' && 'message' in error
      ? String(error.message)
      : error
        ? 'Failed to load onboarding status'
        : null

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
            dispatch(appApi.util.invalidateTags(['OnboardingStatus']))
          }
        )

        unlistenCompleted = await listen<OnboardingCompletedEvent>('onboarding-completed', () => {
          console.log('Onboarding completed event received')
          dispatch(appApi.util.invalidateTags(['OnboardingStatus']))
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
  }, [dispatch])

  return (
    <OnboardingContext.Provider
      value={{ status: data ?? null, loading: isLoading, error: errorMessage, refreshStatus }}
    >
      {children}
    </OnboardingContext.Provider>
  )
}
