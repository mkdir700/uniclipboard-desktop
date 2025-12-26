export interface ThemeColorOption {
  name: string
  color: string
}

export const THEME_COLORS: ThemeColorOption[] = [
  { name: 'zinc', color: '#52525b' },
  { name: 'catppuccin', color: '#cba6f7' },
  { name: 't3chat', color: '#a3004c' },
  { name: 'claude', color: '#d97757' },
]

export const DEFAULT_THEME_COLOR = 'zinc'
