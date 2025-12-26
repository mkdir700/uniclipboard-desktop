/**
 * 设备连接相关 API
 *
 * 提供手动连接设备、获取网络接口等功能
 */

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

/**
 * 网络接口信息
 */
export interface NetworkInterface {
  /** 接口名称 (如 "en0", "Wi-Fi", "以太网") */
  name: string
  /** IP 地址 */
  ip: string
  /** 是否为回环地址 */
  is_loopback: boolean
  /** 是否为 IPv4 */
  is_ipv4: boolean
}

/**
 * 手动连接请求
 */
export interface ManualConnectionRequest {
  /** 目标设备 IP 地址 */
  ip: string
  /** 目标设备端口 */
  port: number
}

/**
 * 手动连接响应
 */
export interface ManualConnectionResponse {
  /** 是否成功 */
  success: boolean
  /** 设备 ID（连接成功时返回） */
  device_id?: string
  /** 响应消息 */
  message: string
}

/**
 * 连接请求信息（发送给接收方）
 */
export interface ConnectionRequestInfo {
  /** 请求方设备 ID */
  requester_device_id: string
  /** 请求方 IP 地址 */
  requester_ip: string
  /** 请求方设备别名（可选） */
  requester_alias?: string
  /** 请求方平台（可选） */
  requester_platform?: string
}

/**
 * 连接请求决策（前端用户确认）
 */
export interface ConnectionRequestDecision {
  /** 是否接受连接 */
  accept: boolean
  /** 请求方设备 ID */
  requester_device_id: string
}

/**
 * 连接响应事件数据
 */
export interface ConnectionResponseEventData {
  /** 是否接受连接 */
  accepted: boolean
  /** 响应方设备 ID */
  responder_device_id: string
  /** 响应方 IP 地址（可选） */
  responder_ip?: string
  /** 响应方设备别名（可选） */
  responder_alias?: string
}

/**
 * 连接状态
 */
export type ConnectionStatus =
  | 'idle' // 空闲状态
  | 'connecting' // 连接中
  | 'awaiting_response' // 等待对方确认
  | 'connected' // 已连接
  | 'failed' // 失败

/**
 * 连接状态信息
 */
export interface ConnectionState {
  /** 状态 */
  status: ConnectionStatus
  /** 消息 */
  message?: string
  /** 设备 ID */
  device_id?: string
  /** 是否可以重试 */
  canRetry?: boolean
}

/**
 * 获取所有本地网络接口
 *
 * 返回所有非回环的 IPv4 地址，用于显示本机可用的网络接口
 */
export async function getLocalNetworkInterfaces(): Promise<NetworkInterface[]> {
  try {
    return await invoke<NetworkInterface[]>('get_local_network_interfaces')
  } catch (error) {
    console.error('Failed to get local network interfaces:', error)
    throw error
  }
}

/**
 * 手动连接到指定设备
 *
 * 发起连接请求到指定 IP 和端口的设备
 *
 * @param request 连接请求参数
 * @returns 连接响应
 */
export async function connectToDeviceManual(
  request: ManualConnectionRequest
): Promise<ManualConnectionResponse> {
  try {
    return await invoke<ManualConnectionResponse>('connect_to_device_manual', {
      request,
    })
  } catch (error) {
    console.error('Failed to connect to device:', error)
    throw error
  }
}

/**
 * 响应连接请求（接受或拒绝）
 *
 * 当收到其他设备的连接请求时，用户可以接受或拒绝
 *
 * @param decision 连接请求决策
 * @returns 连接响应
 */
export async function respondToConnectionRequest(
  decision: ConnectionRequestDecision
): Promise<ManualConnectionResponse> {
  try {
    return await invoke<ManualConnectionResponse>('respond_to_connection_request', {
      decision,
    })
  } catch (error) {
    console.error('Failed to respond to connection request:', error)
    throw error
  }
}

/**
 * 取消待处理的连接请求
 */
export async function cancelConnectionRequest(): Promise<void> {
  try {
    await invoke('cancel_connection_request')
  } catch (error) {
    console.error('Failed to cancel connection request:', error)
    throw error
  }
}

/**
 * 监听连接请求事件
 *
 * 当有设备请求连接时，会触发此事件
 *
 * @param callback 事件回调函数
 * @returns 取消监听的函数
 */
export async function onConnectionRequest(
  callback: (request: ConnectionRequestInfo) => void
): Promise<() => void> {
  try {
    // 先调用后端命令启动监听
    await invoke('listen_connection_request')

    // 然后监听 Tauri 事件
    const unlisten = await listen<ConnectionRequestInfo>('connection-request', event => {
      callback(event.payload)
    })

    // 返回一个清理函数，同时停止后端监听和前端监听
    return async () => {
      unlisten()
      try {
        await invoke('stop_listen_connection_request')
      } catch (e) {
        console.error('Failed to stop listening to connection-request:', e)
      }
    }
  } catch (error) {
    console.error('Failed to setup connection-request listener:', error)
    return () => {}
  }
}

/**
 * 监听连接响应事件
 *
 * 当对方响应连接请求时，会触发此事件
 *
 * @param callback 事件回调函数
 * @returns 取消监听的函数
 */
export async function onConnectionResponse(
  callback: (response: ConnectionResponseEventData) => void
): Promise<() => void> {
  try {
    // 先调用后端命令启动监听
    await invoke('listen_connection_response')

    // 然后监听 Tauri 事件
    const unlisten = await listen<ConnectionResponseEventData>('connection-response', event => {
      callback(event.payload)
    })

    // 返回一个清理函数，同时停止后端监听和前端监听
    return async () => {
      unlisten()
      try {
        await invoke('stop_listen_connection_response')
      } catch (e) {
        console.error('Failed to stop listening to connection-response:', e)
      }
    }
  } catch (error) {
    console.error('Failed to setup connection-response listener:', error)
    return () => {}
  }
}
