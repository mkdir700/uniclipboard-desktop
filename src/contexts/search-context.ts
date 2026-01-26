import { createContext, useContext } from 'react'

export interface SearchContextType {
  searchValue: string
  setSearchValue: (value: string) => void
}

export const SearchContext = createContext<SearchContextType | undefined>(undefined)

export const useSearch = () => {
  const context = useContext(SearchContext)
  if (context === undefined) {
    throw new Error('useSearch must be used within a SearchProvider')
  }
  return context
}
