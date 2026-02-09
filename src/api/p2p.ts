/**
 * P2P device discovery and pairing API
 *
 * 提供 libp2p 设备发现、配对和剪贴板同步功能
 */

import { listen } from '@tauri-apps/api/event'
import { invokeWithTrace } from '@/lib/tauri-command'

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
 * 本地设备信息
 */
export interface LocalDeviceInfo {
  /** Peer ID */
  peerId: string
  /** Device name */
  deviceName: string
}

/**
 * 已配对的设备信息
 */
export interface PairedPeer {
  /** Peer ID */
  peerId: string
  /** Device name */
  deviceName: string
  /** Shared secret (encrypted) */
  sharedSecret: number[]
  /** Pairing timestamp (ISO 8601) */
  pairedAt: string
  /** Last seen timestamp (ISO 8601) */
  lastSeen: string | null
  /** Last known addresses */
  lastKnownAddresses: string[]
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

export type P2PPairingVerificationKind =
  | 'request'
  | 'verification'
  | 'verifying'
  | 'complete'
  | 'failed'

/**
 * P2P 配对验证事件数据
 */
export interface P2PPairingVerificationEvent {
  /** Session ID */
  sessionId: string
  /** Event kind */
  kind: P2PPairingVerificationKind
  /** Peer ID */
  peerId?: string
  /** Device name */
  deviceName?: string
  /** Verification code (short code) */
  code?: string
  /** Local fingerprint */
  localFingerprint?: string
  /** Peer fingerprint */
  peerFingerprint?: string
  /** Error message */
  error?: string
}

/**
 * P2P 设备连接状态变化事件数据
 */
export interface P2PPeerConnectionEvent {
  /** Peer ID */
  peerId: string
  /** Device name (may be null for disconnect) */
  deviceName?: string | null
  /** Connection status */
  connected: boolean
}

/**
 * P2P 设备名称更新事件数据
 */
export interface P2PPeerNameUpdatedEvent {
  /** Peer ID */
  peerId: string
  /** Device name */
  deviceName: string
}

/**
 * P2P 设备发现状态变化事件数据
 */
export interface P2PPeerDiscoveryChangedEvent {
  /** Peer ID */
  peerId: string
  /** Device name (may be null) */
  deviceName?: string | null
  /** Discovered addresses snapshot */
  addresses: string[]
  /** true=discovered, false=lost */
  discovered: boolean
}

/**
 * Space access completion event payload.
 */
export interface SpaceAccessCompletedEvent {
  /** Session ID for idempotency and dedupe */
  sessionId: string
  /** Peer ID */
  peerId: string
  /** Whether access succeeded */
  success: boolean
  /** Optional reason when failed */
  reason?: string
  /** Event timestamp */
  ts: number
}

/**
 * 获取本地 Peer ID
 */
export async function getLocalPeerId(): Promise<string> {
  try {
    return await invokeWithTrace<string>('get_local_peer_id')
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
    return await invokeWithTrace<P2PPeerInfo[]>('get_p2p_peers')
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
    return await invokeWithTrace<P2PPairingResponse>('initiate_p2p_pairing', {
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
    await invokeWithTrace('verify_p2p_pairing_pin', {
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
    await invokeWithTrace('reject_p2p_pairing', {
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
    await invokeWithTrace('unpair_p2p_device', {
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
    await invokeWithTrace('accept_p2p_pairing', {
      sessionId,
    })
  } catch (error) {
    console.error('Failed to accept P2P pairing:', error)
    throw error
  }
}

/**
 * 监听 P2P 配对验证事件
 */
export async function onP2PPairingVerification(
  callback: (event: P2PPairingVerificationEvent) => void
): Promise<() => void> {
  try {
    const unlisten = await listen<P2PPairingVerificationEvent>(
      'p2p-pairing-verification',
      event => {
        callback(event.payload)
      }
    )

    return () => {
      unlisten()
    }
  } catch (error) {
    console.error('Failed to setup P2P pairing verification listener:', error)
    return () => {}
  }
}

/**
 * 监听 P2P 设备连接状态变化事件
 */
export async function onP2PPeerConnectionChanged(
  callback: (event: P2PPeerConnectionEvent) => void
): Promise<() => void> {
  try {
    const unlisten = await listen<P2PPeerConnectionEvent>('p2p-peer-connection-changed', event => {
      callback(event.payload)
    })

    return () => {
      unlisten()
    }
  } catch (error) {
    console.error('Failed to setup P2P peer connection changed listener:', error)
    return () => {}
  }
}

/**
 * 监听 P2P 设备名称更新事件
 */
export async function onP2PPeerNameUpdated(
  callback: (event: P2PPeerNameUpdatedEvent) => void
): Promise<() => void> {
  try {
    const unlisten = await listen<P2PPeerNameUpdatedEvent>('p2p-peer-name-updated', event => {
      callback(event.payload)
    })

    return () => {
      unlisten()
    }
  } catch (error) {
    console.error('Failed to setup P2P peer name updated listener:', error)
    return () => {}
  }
}

/**
 * 监听 P2P 设备发现状态变化事件
 */
export async function onP2PPeerDiscoveryChanged(
  callback: (event: P2PPeerDiscoveryChangedEvent) => void
): Promise<() => void> {
  try {
    const unlisten = await listen<P2PPeerDiscoveryChangedEvent>(
      'p2p-peer-discovery-changed',
      event => {
        callback(event.payload)
      }
    )

    return () => {
      unlisten()
    }
  } catch (error) {
    console.error('Failed to setup P2P peer discovery changed listener:', error)
    return () => {}
  }
}

/**
 * 监听 Space 访问完成事件（带会话幂等过滤与去重）
 */
export async function onSpaceAccessCompleted(
  callback: (event: SpaceAccessCompletedEvent) => void
): Promise<() => void> {
  try {
    let activeSessionId: string | null = null
    const seenEventKeys = new Set<string>()

    const unlisten = await listen<SpaceAccessCompletedEvent>(
      'p2p-space-access-completed',
      event => {
        const payload = event.payload

        if (!payload.sessionId) {
          return
        }

        if (activeSessionId === null) {
          activeSessionId = payload.sessionId
        }

        if (payload.sessionId !== activeSessionId) {
          return
        }

        const dedupeKey = `${payload.sessionId}:${payload.peerId}:${payload.success}:${payload.reason ?? ''}:${payload.ts}`
        if (seenEventKeys.has(dedupeKey)) {
          return
        }
        seenEventKeys.add(dedupeKey)

        callback(payload)

        activeSessionId = null
        seenEventKeys.clear()
      }
    )

    return () => {
      unlisten()
    }
  } catch (error) {
    console.error('Failed to setup space access completed listener:', error)
    return () => {}
  }
}

/**
 * 获取本地设备信息
 */
export async function getLocalDeviceInfo(): Promise<LocalDeviceInfo> {
  try {
    return await invokeWithTrace<LocalDeviceInfo>('get_local_device_info')
  } catch (error) {
    console.error('Failed to get local device info:', error)
    throw error
  }
}

/**
 * 获取已配对的设备列表
 */
export async function getPairedPeers(): Promise<PairedPeer[]> {
  try {
    return await invokeWithTrace<PairedPeer[]>('get_paired_peers')
  } catch (error) {
    console.error('Failed to get paired peers:', error)
    throw error
  }
}

/**
 * 获取已配对的设备列表（带连接状态）
 */
export async function getPairedPeersWithStatus(): Promise<PairedPeer[]> {
  try {
    return await invokeWithTrace<PairedPeer[]>('get_paired_peers_with_status')
  } catch (error) {
    console.error('Failed to get paired peers with status:', error)
    throw error
  }
}
