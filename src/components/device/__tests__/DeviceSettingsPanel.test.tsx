import { render, screen } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import DeviceSettingsPanel from '../DeviceSettingsPanel'

vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, className, ...props }: any) => (
      <div className={className} {...props}>
        {children}
      </div>
    ),
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}))

describe('DeviceSettingsPanel', () => {
  const defaultProps = {
    deviceId: 'test-device-id',
    deviceName: 'Test Device',
  }

  it('renders sync rules section', () => {
    render(<DeviceSettingsPanel {...defaultProps} />)
    expect(screen.getByText('同步设置')).toBeDefined()
    expect(screen.getByText('自动同步')).toBeDefined()
    expect(screen.getByText('同步文本')).toBeDefined()
  })

  it('renders permissions section', () => {
    render(<DeviceSettingsPanel {...defaultProps} />)
    expect(screen.getByText('访问权限')).toBeDefined()
    expect(screen.getByText('读取剪贴板')).toBeDefined()
    expect(screen.getByText('写入剪贴板')).toBeDefined()
  })
})
