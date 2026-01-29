import { createContext } from 'react'
import type { P2PPairingVerificationEvent } from '@/api/p2p'

/**
 * P2P context type definition.
 */
export interface P2PContextType {
  // Receiver pairing request
  pendingRequest: (P2PPairingVerificationEvent & { kind: 'request' }) | null
  showRequestDialog: boolean
  acceptRequest: () => Promise<void>
  rejectRequest: () => Promise<void>

  // PIN verification
  showPinDialog: boolean
  pinData: (P2PPairingVerificationEvent & { kind: 'verification' }) | null
  verifyPin: (matches: boolean) => Promise<void>
  closePinDialog: () => void
}

/**
 * P2P context object.
 */
export const P2PContext = createContext<P2PContextType | undefined>(undefined)
