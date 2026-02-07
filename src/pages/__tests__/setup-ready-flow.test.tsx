import { render, screen, act } from '@testing-library/react'
import type { HTMLAttributes, ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { getSetupState } from '@/api/setup'
import SetupPage from '@/pages/SetupPage'

if (
  typeof globalThis.localStorage === 'undefined' ||
  typeof globalThis.localStorage.getItem !== 'function'
) {
  const store = new Map<string, string>()
  Object.defineProperty(globalThis, 'localStorage', {
    value: {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => {
        store.set(key, value)
      },
      removeItem: (key: string) => {
        store.delete(key)
      },
      clear: () => {
        store.clear()
      },
    },
    configurable: true,
  })
}

if (typeof globalThis.navigator === 'undefined') {
  Object.defineProperty(globalThis, 'navigator', {
    value: { language: 'en-US' },
    configurable: true,
  })
} else if (!('language' in globalThis.navigator)) {
  Object.defineProperty(globalThis.navigator, 'language', {
    value: 'en-US',
    configurable: true,
  })
}

const loadI18n = await import('@/i18n')
const i18n = loadI18n.default

vi.mock('@/api/setup', () => ({
  getSetupState: vi.fn(),
  startNewSpace: vi.fn(),
  startJoinSpace: vi.fn(),
  selectJoinPeer: vi.fn(),
  submitPassphrase: vi.fn(),
  verifyPassphrase: vi.fn(),
  cancelSetup: vi.fn(),
}))

const navigateMock = vi.fn()
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

describe('setup-ready-flow', () => {
  beforeEach(async () => {
    await i18n.changeLanguage('zh-CN')
    vi.mocked(getSetupState).mockReset()
    navigateMock.mockReset()
  })

  it('renders SetupDoneStep when setup state is Completed and allows entering app', async () => {
    const onComplete = vi.fn()
    vi.mocked(getSetupState).mockResolvedValue('Completed')

    render(<SetupPage onCompleteSetup={onComplete} />)

    expect(await screen.findByText('初始化完成')).toBeInTheDocument()
    const enterButton = await screen.findByRole('button', { name: '进入 UniClipboard' })

    await act(async () => {
      enterButton.click()
    })

    expect(onComplete).toHaveBeenCalledTimes(1)
    expect(navigateMock).toHaveBeenCalledWith('/', { replace: true })
  })
})
