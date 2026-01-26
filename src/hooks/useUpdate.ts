import { useContext } from 'react'
import { UpdateContext, type UpdateContextType } from '@/contexts/update-context'

export const useUpdate = (): UpdateContextType => {
  const context = useContext(UpdateContext)

  if (context === undefined) {
    throw new Error('useUpdate必须在UpdateProvider内部使用')
  }

  return context
}
