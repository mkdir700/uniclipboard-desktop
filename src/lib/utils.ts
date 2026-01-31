import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export const isMacPlatform = () => {
  if (typeof navigator === 'undefined') return false
  const userAgent = navigator.userAgent.toLowerCase()
  const platform = (navigator as unknown as { userAgentData?: { platform?: string } }).userAgentData
    ?.platform
  return userAgent.includes('mac') || platform?.toLowerCase() === 'mac'
}

export const isWindowsPlatform = () => {
  if (typeof navigator === 'undefined') return false
  const userAgent = navigator.userAgent.toLowerCase()
  const platform = (navigator as unknown as { userAgentData?: { platform?: string } }).userAgentData
    ?.platform
  return userAgent.includes('windows') || platform?.toLowerCase() === 'windows'
}

export const formatPeerIdForDisplay = (peerId?: string | null, suffixLength = 8) => {
  if (!peerId) return ''
  if (peerId.length <= suffixLength) return peerId
  return `${peerId.slice(-suffixLength)}`
}
