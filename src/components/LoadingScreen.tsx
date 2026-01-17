import { useTranslation } from 'react-i18next'

// Placeholder logo component - replace with actual Logo if available
const Logo = () => (
  <div className="w-20 h-20 bg-muted rounded-lg flex items-center justify-center shadow-lg">
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
      {/* Animated logo container with multiple effects */}
      <div className="relative">
        {/* Outer glow ring */}
        <div className="absolute inset-0 -m-4 rounded-full bg-primary/20 blur-xl animate-pulse" />

        {/* Logo with float animation */}
        <div className="relative animate-float">
          <div className="animate-pulse-slow">
            <Logo />
          </div>

          {/* Shine effect overlay */}
          <div className="absolute inset-0 rounded-lg overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-tr from-transparent via-white/10 to-transparent translate-x-[-100%] animate-shine" />
          </div>
        </div>
      </div>

      {/* Status text with fade-in animation */}
      <div className="mt-12 text-sm text-muted-foreground animate-fade-in">
        {t('loading.initializing')}
      </div>

      {/* Loading dots */}
      <div className="mt-4 flex gap-2">
        <div className="w-2 h-2 rounded-full bg-primary/60 animate-bounce [animation-delay:-0.3s]" />
        <div className="w-2 h-2 rounded-full bg-primary/60 animate-bounce [animation-delay:-0.15s]" />
        <div className="w-2 h-2 rounded-full bg-primary/60 animate-bounce" />
      </div>
    </div>
  )
}
