# LoadingScreen Redesign - Modern Minimal Style

**Date**: 2026-01-18
**Status**: Design
**Author**: Claude

## Overview

Redesign the existing LoadingScreen component with a modern minimal aesthetic, replacing the Logo placeholder with an animated product name "UniClipboard" and elegant pulse-dot animations.

## Design Goals

1. **Modern minimal style** - Clean, spacious, refined animations
2. **Elegant typography** - Product name with staggered letter-by-letter fade-in
3. **Smooth animations** - Gentle pulse waves with rhythmic timing
4. **Theme adaptive** - Works seamlessly in light and dark modes
5. **i18n support** - Status text remains internationalized

## Visual Design

### Layout Structure

```
┌─────────────────────────────────────┐
│                                     │
│         UniClipboard                │  ← Animated product name
│      (letter-by-letter fade-in)     │
│                                     │
│    正在初始化...                     │  ← Status text (delayed fade-in)
│                                     │
│      ●   ●   ●                      │  ← Pulse dots (wave animation)
│                                     │
└─────────────────────────────────────┘
```

### Color Palette

| Element      | Light Mode    | Dark Mode     | Tailwind Class          |
| ------------ | ------------- | ------------- | ----------------------- |
| Background   | White         | Dark gray     | `bg-background`         |
| Product name | Near black    | Near white    | `text-foreground`       |
| Status text  | Medium gray   | Medium gray   | `text-muted-foreground` |
| Pulse dots   | Primary color | Primary color | `bg-primary/70`         |

### Typography

**Product Name "UniClipboard"**:

- Size: `text-5xl` (48px) / `text-4xl` (32px) on mobile
- Weight: `font-light` (300)
- Letter spacing: `tracking-wide` (0.025em)
- Color: `text-foreground`

**Status Text**:

- Size: `text-sm` (14px)
- Weight: `font-medium` (500)
- Color: `text-muted-foreground`
- Content: `{t('loading.initializing')}`

## Animations

### 1. Fade-In-Up (Product Name Letters)

Each letter animates sequentially with staggered delays:

```css
@keyframes fade-in-up {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-fade-in-up {
  animation: fade-in-up 0.6s cubic-bezier(0.4, 0, 0.2, 1) both;
}
```

**Parameters**:

- Duration: 0.6s per letter
- Delay: 50ms increment per letter
- Total duration: ~1.15s (for "UniClipboard")
- Easing: cubic-bezier(0.4, 0, 0.2, 1) - elegant deceleration

### 2. Pulse-Wave (Loading Dots)

Three dots animate in a wave pattern:

```css
@keyframes pulse-wave {
  0%,
  100% {
    opacity: 0.4;
    transform: scale(0.8);
  }
  50% {
    opacity: 1;
    transform: scale(1.2);
  }
}

.animate-pulse-wave {
  animation: pulse-wave 1.2s ease-in-out infinite;
}
```

**Parameters**:

- Duration: 1.2s per cycle
- Dot 1 delay: -0.4s (starts at 33% of cycle)
- Dot 2 delay: -0.2s (starts at 66% of cycle)
- Dot 3 delay: 0s (starts at beginning)
- Scale: 0.8 ↔ 1.2
- Opacity: 0.4 ↔ 1.0

### Animation Timeline

```
0.00s   → First letter "U" starts fading in
0.05s   → Second letter "n" starts fading in
0.10s   → Third letter "i" starts fading in
...
1.15s   → Last letter "d" finishes fading in
1.35s   → Status text starts fading in (300ms duration)
1.65s   → Status text fully visible + pulse dots start
```

## Component Implementation

### File: `src/components/LoadingScreen.tsx`

```tsx
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
```

### CSS Animations

Add to `src/index.css` or `src/globals.css`:

```css
/* Product name letter fade-in with upward motion */
@keyframes fade-in-up {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-fade-in-up {
  animation: fade-in-up 0.6s cubic-bezier(0.4, 0, 0.2, 1) both;
}

/* Pulse wave for loading dots */
@keyframes pulse-wave {
  0%,
  100% {
    opacity: 0.4;
    transform: scale(0.8);
  }
  50% {
    opacity: 1;
    transform: scale(1.2);
  }
}

.animate-pulse-wave {
  animation: pulse-wave 1.2s ease-in-out infinite;
}
```

## Key Changes from Original Design

| Aspect            | Original                 | New Design               |
| ----------------- | ------------------------ | ------------------------ |
| Logo              | UC placeholder box       | "UniClipboard" text      |
| Logo animation    | Pulse + float + shine    | Letter-by-letter fade-in |
| Loading dots      | Bounce animation         | Wave pulse animation     |
| Visual complexity | Multiple layered effects | Clean, minimal           |
| Spacing           | Compact (mt-12, mt-4)    | Generous (mb-8, mt-6)    |
| Animation timing  | All immediate            | Staggered, choreographed |

## Design Rationale

### Why Letter-by-Letter Animation?

- **Premium feel**: Sequential animation shows attention to detail
- **Brand focus**: Product name becomes the hero element
- **Rhythm**: Creates a pleasing visual cadence
- **Scalability**: Works for any brand name length

### Why Wave Pulse Dots?

- **Calmness**: Gentle wave motion is less urgent than bounce
- **Continuity**: Infinite loop suggests ongoing work without anxiety
- **Harmony**: Scales and opacity sync for organic feel
- **Directionality**: Wave pattern implies forward progress

### Why Modern Minimal Style?

- **Timelessness**: Avoids trendy effects that date quickly
- **Performance**: Simple animations run smoothly on all devices
- **Accessibility**: Clean, high-contrast design is easier to read
- **Brand alignment**: Professional, trustworthy appearance

## Accessibility

- `role="status"` - Identifies as a status message container
- `aria-live="polite"` - Announces changes without interrupting
- High contrast ratios in both light and dark modes
- No flashing or strobing effects (photosensitive-friendly)

## Testing Checklist

- [ ] Verify animations play smoothly in Chrome, Firefox, Safari, Edge
- [ ] Test light mode theme appearance
- [ ] Test dark mode theme appearance
- [ ] Verify responsive sizing on mobile (320px width and up)
- [ ] Check animation performance with DevTools Performance tab
- [ ] Verify i18n text displays correctly in all configured languages
- [ ] Test fade-out transition when `backend-ready` event fires
- [ ] Verify no layout shift during animations

## Files to Modify

| File                                 | Action                          |
| ------------------------------------ | ------------------------------- |
| `src/components/LoadingScreen.tsx`   | Replace with new implementation |
| `src/index.css` or `src/globals.css` | Add custom CSS animations       |

## Success Criteria

- ✅ Product name "UniClipboard" displays with letter-by-letter animation
- ✅ Status text fades in after product name animation completes
- ✅ Pulse dots animate in wave pattern with proper timing
- ✅ Design works in both light and dark themes
- ✅ Animations are smooth (60fps) on all platforms
- ✅ Component maintains existing props interface (`className`)
- ✅ i18n support preserved for status text
