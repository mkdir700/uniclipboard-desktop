import type { CSSProperties } from 'react'
import type { ToasterProps } from 'sonner'
import { Toaster as SonnerToaster } from 'sonner'
import { useSetting } from '@/hooks/useSetting'

function Toaster({ ...props }: ToasterProps) {
  const { setting } = useSetting()
  const theme = setting?.general.theme || 'system'

  return (
    <SonnerToaster
      theme={theme as ToasterProps['theme']}
      className="toaster group"
      toastOptions={{
        classNames: {
          toast:
            'group toast group-[.toaster]:bg-background group-[.toaster]:text-foreground group-[.toaster]:border-border group-[.toaster]:shadow-lg',
          description: 'group-[.toast]:text-muted-foreground',
          actionButton: 'group-[.toast]:bg-primary group-[.toast]:text-primary-foreground',
          cancelButton: 'group-[.toast]:bg-muted group-[.toast]:text-muted-foreground',
        },
      }}
      style={
        {
          '--normal-bg': 'var(--background)',
          '--normal-text': 'var(--foreground)',
          '--normal-border': 'var(--border)',
          '--success-bg': 'var(--primary)',
          '--success-border': 'var(--primary)',
          '--success-text': 'var(--primary-foreground)',
          '--error-bg': 'var(--destructive)',
          '--error-border': 'var(--destructive)',
          '--error-text': 'var(--destructive-foreground)',
          '--warning-bg': 'var(--accent)',
          '--warning-border': 'var(--accent)',
          '--warning-text': 'var(--accent-foreground)',
          '--info-bg': 'var(--primary)',
          '--info-border': 'var(--primary)',
          '--info-text': 'var(--primary-foreground)',
          '--border-radius': 'var(--radius)',
        } as CSSProperties
      }
      {...props}
    />
  )
}

export { Toaster }
