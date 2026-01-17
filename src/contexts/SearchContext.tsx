import React, { createContext, useContext, useState, useCallback } from 'react'

interface SearchContextType {
  searchValue: string
  setSearchValue: (value: string) => void
}

const SearchContext = createContext<SearchContextType | undefined>(undefined)

export const SearchProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
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

export const useSearch = () => {
  const context = useContext(SearchContext)
  if (context === undefined) {
    throw new Error('useSearch must be used within a SearchProvider')
  }
  return context
}
