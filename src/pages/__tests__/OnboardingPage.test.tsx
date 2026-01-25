import { render, screen, act } from '@testing-library/react'
import type { HTMLAttributes, ReactNode } from 'react'
import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'
import OnboardingPage from '@/pages/OnboardingPage'

const navigateMock = vi.hoisted(() => vi.fn())
const refreshStatusMock = vi.hoisted(() => vi.fn())
const completeOnboardingMock = vi.hoisted(() => vi.fn())
const toastErrorMock = vi.hoisted(() => vi.fn())

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom')
  return {
    ...actual,
    useNavigate: () => navigateMock,
  }
})

vi.mock('framer-motion', () => ({
  AnimatePresence: ({ children }: { children: ReactNode }) => <>{children}</>,
  motion: new Proxy(
    {},
    {
      get: () => (props: HTMLAttributes<HTMLDivElement>) => <div {...props} />,
    }
  ),
}))

vi.mock('@/contexts/OnboardingContext', () => ({
  useOnboarding: () => ({
    status: {
      has_completed: false,
      encryption_password_set: true,
      device_registered: false,
    },
    refreshStatus: refreshStatusMock,
  }),
}))

vi.mock('@/api/onboarding', () => ({
  completeOnboarding: completeOnboardingMock,
  setupEncryptionPassword: vi.fn(),
}))

vi.mock('sonner', () => ({
  toast: {
    error: toastErrorMock,
  },
}))

describe('OnboardingPage', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    navigateMock.mockReset()
    toastErrorMock.mockReset()
    refreshStatusMock.mockReset()
    completeOnboardingMock.mockReset()
    completeOnboardingMock.mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('shows error when refreshStatus rejects in completion timeout', async () => {
    refreshStatusMock.mockRejectedValue(new Error('timeout failure'))

    render(<OnboardingPage />)

    await act(async () => {})

    await act(async () => {
      vi.advanceTimersByTime(5000)
      await vi.runAllTimersAsync()
    })

    await act(async () => {})

    expect(refreshStatusMock).toHaveBeenCalled()
    expect(screen.getByText('出现错误')).toBeInTheDocument()
    expect(toastErrorMock).toHaveBeenCalledWith('timeout failure')
  })
})
