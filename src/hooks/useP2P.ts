import { useContext } from 'react'
import { P2PContext } from '@/types/p2p'

/**
 * Hook to use the P2P context.
 * @throws {Error} If used outside of P2PProvider.
 */
export const useP2P = () => {
  const context = useContext(P2PContext)
  if (!context) {
    throw new Error('useP2P must be used within P2PProvider')
  }
  return context
}
