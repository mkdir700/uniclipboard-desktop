import type { Update } from '@tauri-apps/plugin-updater'
import { render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import Sidebar from '@/components/layout/Sidebar'
import { SettingContext } from '@/contexts/setting-context'
import { UpdateContext } from '@/contexts/update-context'
import type { Settings } from '@/types/setting'

const baseSetting: Settings = {
  schema_version: 1,
  general: {
    auto_start: false,
    silent_start: false,
    auto_check_update: true,
    theme: 'system',
    theme_color: null,
    language: 'en-US',
    device_name: 'Test Device',
  },
  sync: {
    auto_sync: true,
    sync_frequency: 'realtime',
    content_types: {
      text: true,
      image: true,
      link: true,
      file: true,
      code_snippet: true,
      rich_text: true,
    },
    max_file_size_mb: 10,
  },
  retention_policy: {
    enabled: false,
    rules: [],
    skip_pinned: false,
    evaluation: 'any_match',
  },
  security: {
    encryption_enabled: false,
    passphrase_configured: false,
    auto_unlock_enabled: false,
  },
}

describe('Sidebar update indicator', () => {
  it('shows update icon when updater returns update info', async () => {
    const updateInfo = {
      version: '0.1.1',
      currentVersion: '0.1.0',
      date: '2026-01-25T00:00:00Z',
      body: 'Bug fixes',
      downloadAndInstall: vi.fn(),
      close: vi.fn(),
    } as unknown as Update

    render(
      <SettingContext.Provider
        value={{
          setting: baseSetting,
          loading: false,
          error: null,
          updateSetting: vi.fn(),
          updateGeneralSetting: vi.fn(),
          updateSyncSetting: vi.fn(),
          updateSecuritySetting: vi.fn(),
          updateRetentionPolicy: vi.fn(),
        }}
      >
        <UpdateContext.Provider
          value={{
            updateInfo,
            isCheckingUpdate: false,
            checkForUpdates: vi.fn(),
          }}
        >
          <MemoryRouter>
            <Sidebar />
          </MemoryRouter>
        </UpdateContext.Provider>
      </SettingContext.Provider>
    )

    await waitFor(() => {
      expect(screen.getByLabelText(/update available/i)).toBeInTheDocument()
    })
  })

  it('hides update icon when there is no update info', () => {
    render(
      <SettingContext.Provider
        value={{
          setting: baseSetting,
          loading: false,
          error: null,
          updateSetting: vi.fn(),
          updateGeneralSetting: vi.fn(),
          updateSyncSetting: vi.fn(),
          updateSecuritySetting: vi.fn(),
          updateRetentionPolicy: vi.fn(),
        }}
      >
        <UpdateContext.Provider
          value={{
            updateInfo: null,
            isCheckingUpdate: false,
            checkForUpdates: vi.fn(),
          }}
        >
          <MemoryRouter>
            <Sidebar />
          </MemoryRouter>
        </UpdateContext.Provider>
      </SettingContext.Provider>
    )

    expect(screen.queryByLabelText(/update available/i)).not.toBeInTheDocument()
  })
})
