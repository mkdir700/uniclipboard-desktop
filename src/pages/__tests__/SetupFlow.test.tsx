import { render, screen, act } from '@testing-library/react'
import type { HTMLAttributes, ReactNode } from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { getSetupState, startNewSpace } from '@/api/setup'
import i18n from '@/i18n'
import SetupPage from '@/pages/SetupPage'

// Mock the API
vi.mock('@/api/setup', () => ({
  getSetupState: vi.fn(),
  startNewSpace: vi.fn(),
  startJoinSpace: vi.fn(),
  selectJoinPeer: vi.fn(),
  submitPassphrase: vi.fn(),
  verifyPassphrase: vi.fn(),
  cancelSetup: vi.fn(),
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

describe('Setup flow', () => {
  beforeEach(async () => {
    await i18n.changeLanguage('zh-CN')
  })

  it('renders welcome step for SetupState.Welcome', async () => {
    // mock getSetupState() to return 'Welcome'
    vi.mocked(getSetupState).mockResolvedValue('Welcome')

    render(<SetupPage />)

    expect(await screen.findByText('欢迎使用 UniClipboard')).toBeInTheDocument()
    expect(screen.getByText('选择一种方式开始设置你的加密空间')).toBeInTheDocument()
    expect(screen.getByText('创建新的加密空间')).toBeInTheDocument()

    await act(async () => {
      await i18n.changeLanguage('en-US')
    })

    expect(await screen.findByText('Welcome to UniClipboard')).toBeInTheDocument()
  })

  it('shows passphrase mismatch error text', async () => {
    // mock getSetupState() to return CreateSpaceInputPassphrase with error
    vi.mocked(getSetupState).mockResolvedValue({
      CreateSpaceInputPassphrase: { error: 'PassphraseMismatch' },
    })

    render(<SetupPage />)

    // Wait for the error message to appear
    expect(await screen.findByText('两次输入不一致，请重新确认。')).toBeInTheDocument()
  })

  it('starts new space when clicking create CTA', async () => {
    vi.mocked(getSetupState).mockResolvedValue('Welcome')
    vi.mocked(startNewSpace).mockResolvedValue({
      CreateSpaceInputPassphrase: { error: null },
    })
    render(<SetupPage />)

    const createBtn = await screen.findByText('创建新的加密空间')
    await act(async () => {
      createBtn.click()
    })

    expect(startNewSpace).toHaveBeenCalled()
  })
})
