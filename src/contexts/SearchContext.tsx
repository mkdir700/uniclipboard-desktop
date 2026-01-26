import { useCallback, useState, type ReactNode } from 'react'
import { SearchContext } from './search-context'

export const SearchProvider = ({ children }: { children: ReactNode }) => {
  const [searchValue, setSearchValue] = useState('')

  const handleSetSearchValue = useCallback((value: string) => {
    setSearchValue(value)
  }, [])

  return (
    <SearchContext.Provider value={{ searchValue, setSearchValue: handleSetSearchValue }}>
      {children}
    </SearchContext.Provider>
  )
}
