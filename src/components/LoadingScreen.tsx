import { useTranslation } from 'react-i18next'

// Placeholder logo component - replace with actual Logo if available
const Logo = () => (
  <div className="w-20 h-20 bg-muted rounded-lg flex items-center justify-center">
    <span className="text-2xl font-bold">UC</span>
  </div>
)

interface LoadingScreenProps {
  className?: string
}

export const LoadingScreen: React.FC<LoadingScreenProps> = ({ className = '' }) => {
  const { t } = useTranslation()

  return (
    <div
      className={`h-screen w-screen flex flex-col items-center justify-center bg-background ${className}`}
    >
      {/* Logo with pulse animation */}
      <div className="animate-pulse opacity-70">
        <Logo />
      </div>

      {/* Status text */}
      <div className="mt-8 text-sm text-muted-foreground">{t('loading.initializing')}</div>
    </div>
  )
}
