import { render, screen } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { getSetupState, dispatchSetupEvent } from '@/api/onboarding'
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
  AnimatePresence: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  motion: new Proxy(
    {},
    {
      get: () => (props: React.HTMLAttributes<HTMLDivElement>) => <div {...props} />,
    }
  ),
}))

describe('Onboarding flow', () => {
  it('renders welcome step for SetupState.Welcome', async () => {
    // mock getSetupState() to return 'Welcome'
    vi.mocked(getSetupState).mockResolvedValue('Welcome')

    render(<OnboardingPage />)

    // Wait for the state to load and render
    // We expect to see the new Welcome step content
    // Based on the plan, we should look for "欢迎使用 UniClipboard" and "创建新的加密空间"
    // But currently OnboardingPage renders the old UI.
    // So this test should fail because the old UI has "欢迎来到 UniClipboard" (slightly different)
    // or we can look for the specific new buttons.

    // Let's look for the new text from the plan/spec (implied)
    expect(await screen.findByText('欢迎使用 UniClipboard')).toBeInTheDocument()
    expect(screen.getByText('创建新的加密空间')).toBeInTheDocument()
  })

  it('shows passphrase mismatch error text', async () => {
    // mock getSetupState() to return CreateSpacePassphrase with error
    vi.mocked(getSetupState).mockResolvedValue({
      CreateSpacePassphrase: { error: 'PassphraseMismatch' },
    })

    render(<OnboardingPage />)

    // Wait for the error message to appear
    expect(await screen.findByText('两次输入的密码不一致')).toBeInTheDocument()
  })

  it('dispatches ChooseCreateSpace when clicking create CTA', async () => {
    vi.mocked(getSetupState).mockResolvedValue('Welcome')
    render(<OnboardingPage />)

    const createBtn = await screen.findByText('创建新的加密空间')
    createBtn.click()

    expect(dispatchSetupEvent).toHaveBeenCalledWith('ChooseCreateSpace')
  })
})
