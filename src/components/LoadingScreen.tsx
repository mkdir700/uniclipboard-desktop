import { useTranslation } from 'react-i18next'

interface LoadingScreenProps {
  className?: string
}

// Animated text component for staggered letter animation
const AnimatedText = ({ text, className }: { text: string; className?: string }) => (
  <div className={className}>
    {text.split('').map((char, i) => (
      <span
        key={i}
        className="inline-block animate-fade-in-up"
        style={{ animationDelay: `${i * 0.05}s` }}
      >
        {char}
      </span>
    ))}
  </div>
)

export const LoadingScreen: React.FC<LoadingScreenProps> = ({ className = '' }) => {
  const { t } = useTranslation()

  return (
    <div
      className={`h-screen w-screen flex flex-col items-center justify-center bg-background ${className}`}
      role="status"
      aria-live="polite"
    >
      {/* Product name with letter-by-letter animation */}
      <AnimatedText
        text="UniClipboard"
        className="text-4xl md:text-5xl font-light tracking-wide text-foreground mb-8"
      />

      {/* Status text with delayed fade-in */}
      <div
        className="text-sm text-muted-foreground font-medium animate-fade-in"
        style={{ animationDelay: '1.35s' }}
      >
        {t('loading.initializing')}
      </div>

      {/* Pulse dots with wave animation */}
      <div className="mt-6 flex gap-2">
        <div
          className="w-2 h-2 rounded-full bg-primary/70 animate-pulse-wave"
          style={{ animationDelay: '-0.4s' }}
        />
        <div
          className="w-2 h-2 rounded-full bg-primary/70 animate-pulse-wave"
          style={{ animationDelay: '-0.2s' }}
        />
        <div
          className="w-2 h-2 rounded-full bg-primary/70 animate-pulse-wave"
          style={{ animationDelay: '0s' }}
        />
      </div>
    </div>
  )
}
