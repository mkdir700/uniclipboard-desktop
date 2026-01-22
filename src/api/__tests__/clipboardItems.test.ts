import { invoke } from '@tauri-apps/api/core'
import { describe, expect, it, vi } from 'vitest'
import { getClipboardItems } from '@/api/clipboardItems'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const invokeMock = vi.mocked(invoke)

describe('getClipboardItems', () => {
  it('将 image/* 条目映射为 image 类型，并优先使用后端返回的 thumbnail_url', async () => {
    invokeMock.mockResolvedValueOnce([
      {
        id: 'entry-1',
        preview: 'Image (123 bytes)',
        has_detail: true,
        size_bytes: 123,
        captured_at: 1,
        content_type: 'image/png',
        is_encrypted: false,
        is_favorited: false,
        updated_at: 1,
        active_time: 1,
        thumbnail_url: 'uc://blob/thumb-1',
      },
    ])

    const items = await getClipboardItems(undefined, 50, 0, undefined)

    expect(items).toHaveLength(1)
    expect(items[0].item.image).toBeTruthy()
    expect(items[0].item.text).toBeFalsy()
    expect(items[0].item.image?.thumbnail).toBe('uc://blob/thumb-1')
  })
})
