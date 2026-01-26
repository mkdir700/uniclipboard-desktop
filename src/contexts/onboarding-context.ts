import { createContext, useContext } from 'react'
import { type OnboardingStatus } from '@/api/onboarding'

export interface OnboardingContextType {
  status: OnboardingStatus | null
  loading: boolean
  error: string | null
  refreshStatus: () => Promise<OnboardingStatus>
}

export const OnboardingContext = createContext<OnboardingContextType | undefined>(undefined)

export const useOnboarding = () => {
  const context = useContext(OnboardingContext)
  if (!context) {
    throw new Error('useOnboarding must be used within OnboardingProvider')
  }
  return context
}
