import { render, screen, act } from '@testing-library/react'
import type { HTMLAttributes, ReactNode } from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { getSetupState, dispatchSetupEvent } from '@/api/onboarding'
import i18n from '@/i18n'
import OnboardingPage from '@/pages/OnboardingPage'

// Mock the API
vi.mock('@/api/onboarding', () => ({
  getSetupState: vi.fn(),
  dispatchSetupEvent: vi.fn(),
  // Keep existing mocks if needed, but for now we focus on the new flow
  getOnboardingState: vi.fn(),
  initializeOnboarding: vi.fn(),
  completeOnboarding: vi.fn(),
  setupEncryptionPassword: vi.fn(),
}))

// Mock the context if it's still used, or we might need to wrap the component
vi.mock('@/contexts/onboarding-context', () => ({
  useOnboarding: () => ({
    status: {
      has_completed: false,
      encryption_password_set: false,
      device_registered: false,
    },
    refreshStatus: vi.fn(),
  }),
}))

// Mock react-router-dom
const navigateMock = vi.fn()
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom')
  return {
    ...actual,
    useNavigate: () => navigateMock,
  }
})

// Mock framer-motion to avoid animation issues in tests
vi.mock('framer-motion', () => ({
  AnimatePresence: ({ children }: { children: ReactNode }) => <>{children}</>,
  motion: new Proxy(
    {},
    {
      get: () => (props: HTMLAttributes<HTMLDivElement>) => <div {...props} />,
    }
  ),
}))

describe('Onboarding flow', () => {
  beforeEach(async () => {
    await i18n.changeLanguage('zh-CN')
  })

  it('renders welcome step for SetupState.Welcome', async () => {
    // mock getSetupState() to return 'Welcome'
    vi.mocked(getSetupState).mockResolvedValue('Welcome')

    render(<OnboardingPage />)

    expect(await screen.findByText('欢迎使用 UniClipboard')).toBeInTheDocument()
    expect(screen.getByText('选择一种方式开始设置你的加密空间')).toBeInTheDocument()
    expect(screen.getByText('创建新的加密空间')).toBeInTheDocument()

    await act(async () => {
      await i18n.changeLanguage('en-US')
    })

    expect(await screen.findByText('Welcome to UniClipboard')).toBeInTheDocument()
  })

  it('shows passphrase mismatch error text', async () => {
    // mock getSetupState() to return CreateSpacePassphrase with error
    vi.mocked(getSetupState).mockResolvedValue({
      CreateSpacePassphrase: { error: 'PassphraseMismatch' },
    })

    render(<OnboardingPage />)

    // Wait for the error message to appear
    expect(await screen.findByText('两次输入不一致，请重新确认。')).toBeInTheDocument()
  })

  it('dispatches ChooseCreateSpace when clicking create CTA', async () => {
    vi.mocked(getSetupState).mockResolvedValue('Welcome')
    vi.mocked(dispatchSetupEvent).mockResolvedValue('Welcome')
    render(<OnboardingPage />)

    const createBtn = await screen.findByText('创建新的加密空间')
    await act(async () => {
      createBtn.click()
    })

    expect(dispatchSetupEvent).toHaveBeenCalledWith('ChooseCreateSpace')
  })
})
