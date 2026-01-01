/**
 * P2P device discovery and pairing API
 *
 * 提供 libp2p 设备发现、配对和剪贴板同步功能
 */

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

/**
 * P2P 设备信息
 */
export interface P2PPeerInfo {
  /** Peer ID (libp2p identifier) */
  peerId: string
  /** Device name (may be null if not yet discovered) */
  deviceName?: string | null
  /** Addresses */
  addresses: string[]
  /** Whether this peer is paired */
  isPaired: boolean
  /** Connection status */
  connected: boolean
}

/**
 * P2P 配对请求
 */
export interface P2PPairingRequest {
  /** Target peer ID */
  peerId: string
}

/**
 * P2P 配对响应
 */
export interface P2PPairingResponse {
  /** Session ID for this pairing attempt */
  sessionId: string
  /** Whether pairing was initiated successfully */
  success: boolean
  /** Error message if failed */
  error?: string
}

/**
 * P2P PIN 验证请求
 */
export interface P2PPinVerifyRequest {
  /** Session ID */
  sessionId: string
  /** Whether PIN matches */
  pinMatches: boolean
}

/**
 * P2P 配对请求事件数据
 */
export interface P2PPairingRequestEvent {
  /** Session ID */
  sessionId: string
  /** Peer ID of the requester */
  peerId: string
  /** Device name of the requester */
  deviceName?: string
}

/**
 * P2P PIN 就绪事件数据
 */
export interface P2PPinReadyEvent {
  /** Session ID */
  sessionId: string
  /** PIN to verify */
  pin: string
  /** Peer device name */
  peerDeviceName?: string
}

/**
 * P2P 配对完成事件数据
 */
export interface P2PPairingCompleteEvent {
  /** Session ID */
  sessionId: string
  /** Peer ID */
  peerId: string
  /** Device name */
  deviceName: string
}

/**
 * P2P 配对失败事件数据
 */
export interface P2PPairingFailedEvent {
  /** Session ID */
  sessionId: string
  /** Error message */
  error: string
}

/**
 * 获取本地 Peer ID
 */
export async function getLocalPeerId(): Promise<string> {
  try {
    return await invoke<string>('get_local_peer_id')
  } catch (error) {
    console.error('Failed to get local peer ID:', error)
    throw error
  }
}

/**
 * 获取发现的 P2P 设备列表
 */
export async function getP2PPeers(): Promise<P2PPeerInfo[]> {
  try {
    return await invoke<P2PPeerInfo[]>('get_p2p_peers')
  } catch (error) {
    console.error('Failed to get P2P peers:', error)
    throw error
  }
}

/**
 * 发起 P2P 配对请求
 */
export async function initiateP2PPairing(request: P2PPairingRequest): Promise<P2PPairingResponse> {
  try {
    return await invoke<P2PPairingResponse>('initiate_p2p_pairing', {
      request,
    })
  } catch (error) {
    console.error('Failed to initiate P2P pairing:', error)
    throw error
  }
}

/**
 * 验证 PIN 并完成配对
 */
export async function verifyP2PPairingPin(request: P2PPinVerifyRequest): Promise<void> {
  try {
    await invoke('verify_p2p_pairing_pin', {
      request,
    })
  } catch (error) {
    console.error('Failed to verify P2P pairing PIN:', error)
    throw error
  }
}

/**
 * 拒绝 P2P 配对请求
 */
export async function rejectP2PPairing(sessionId: string, peerId: string): Promise<void> {
  try {
    await invoke('reject_p2p_pairing', {
      sessionId,
      peerId,
    })
  } catch (error) {
    console.error('Failed to reject P2P pairing:', error)
    throw error
  }
}

/**
 * 取消 P2P 配对连接
 */
export async function unpairP2PDevice(peerId: string): Promise<void> {
  try {
    await invoke('unpair_p2p_device', {
      peerId,
    })
  } catch (error) {
    console.error('Failed to unpair P2P device:', error)
    throw error
  }
}

/**
 * 接受 P2P 配对请求（接收方）
 */
export async function acceptP2PPairing(sessionId: string): Promise<void> {
  try {
    await invoke('accept_p2p_pairing', {
      sessionId,
    })
  } catch (error) {
    console.error('Failed to accept P2P pairing:', error)
    throw error
  }
}

/**
 * 监听 P2P 配对请求事件
 */
export async function onP2PPairingRequest(
  callback: (request: P2PPairingRequestEvent) => void
): Promise<() => void> {
  try {
    const unlisten = await listen<P2PPairingRequestEvent>('p2p-pairing-request', event => {
      callback(event.payload)
    })

    return () => {
      unlisten()
    }
  } catch (error) {
    console.error('Failed to setup P2P pairing request listener:', error)
    return () => {}
  }
}

/**
 * 监听 P2P PIN 就绪事件
 */
export async function onP2PPinReady(
  callback: (event: P2PPinReadyEvent) => void
): Promise<() => void> {
  try {
    const unlisten = await listen<P2PPinReadyEvent>('p2p-pin-ready', event => {
      callback(event.payload)
    })

    return () => {
      unlisten()
    }
  } catch (error) {
    console.error('Failed to setup P2P PIN ready listener:', error)
    return () => {}
  }
}

/**
 * 监听 P2P 配对完成事件
 */
export async function onP2PPairingComplete(
  callback: (event: P2PPairingCompleteEvent) => void
): Promise<() => void> {
  try {
    const unlisten = await listen<P2PPairingCompleteEvent>('p2p-pairing-complete', event => {
      callback(event.payload)
    })

    return () => {
      unlisten()
    }
  } catch (error) {
    console.error('Failed to setup P2P pairing complete listener:', error)
    return () => {}
  }
}

/**
 * 监听 P2P 配对失败事件
 */
export async function onP2PPairingFailed(
  callback: (event: P2PPairingFailedEvent) => void
): Promise<() => void> {
  try {
    const unlisten = await listen<P2PPairingFailedEvent>('p2p-pairing-failed', event => {
      callback(event.payload)
    })

    return () => {
      unlisten()
    }
  } catch (error) {
    console.error('Failed to setup P2P pairing failed listener:', error)
    return () => {}
  }
}
