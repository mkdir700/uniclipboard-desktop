# LoadingScreen Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the LoadingScreen component with a modern minimal design featuring animated "UniClipboard" product name and elegant pulse-dot loading animation.

**Architecture:** React component with CSS keyframe animations. The component uses a helper `AnimatedText` subcomponent for staggered letter-by-letter fade-in animation. Custom CSS animations are added to the global stylesheet for reuse.

**Tech Stack:** React 18, TypeScript, Tailwind CSS, react-i18next

---

## Task 1: Add CSS Animations to Global Stylesheet

**Files:**

- Modify: `src/styles/globals.css`

**Step 1: Add custom animation keyframes**

Add the following CSS to the END of `src/styles/globals.css` (after line 249):

```css
/* LoadingScreen animations */
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

**What this does:**

- `fade-in-up`: Letters fade in while moving up 8px with elegant easing
- `pulse-wave`: Dots scale and opacity animate in a wave pattern
- Using `cubic-bezier(0.4, 0, 0.2, 1)` for smooth, premium-feeling deceleration

**Step 2: Verify syntax**

Run: `bun run dev`
Expected: Vite dev server starts without CSS syntax errors

**Step 3: Commit**

```bash
git add src/styles/globals.css
git commit -m "feat(styles): add fade-in-up and pulse-wave animations for LoadingScreen

Add custom CSS keyframe animations:
- fade-in-up: staggered letter animation with upward motion
- pulse-wave: loading dots wave pattern animation

These animations will be used by the redesigned LoadingScreen component.

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 2: Replace LoadingScreen Component

**Files:**

- Modify: `src/components/LoadingScreen.tsx` (replace entire file)

**Step 1: Read current implementation**

Run: `cat src/components/LoadingScreen.tsx`
Expected: See current implementation with Logo placeholder and bounce animations

**Step 2: Replace with new implementation**

Replace the ENTIRE content of `src/components/LoadingScreen.tsx` with:

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

**What this does:**

- Removes the Logo placeholder component
- Adds `AnimatedText` helper for letter-by-letter animation
- Displays "UniClipboard" with staggered fade-in-up animation
- Status text fades in after 1.35s delay
- Three pulse dots animate in wave pattern with staggered delays
- Adds accessibility attributes (`role="status"`, `aria-live="polite"`)
- Maintains `className` prop for fade-out transition support

**Step 3: Verify TypeScript compilation**

Run: `bun run build`
Expected: Build succeeds with no TypeScript errors

**Step 4: Commit**

```bash
git add src/components/LoadingScreen.tsx
git commit -m "feat(ui): redesign LoadingScreen with modern minimal style

Replace Logo placeholder with animated product name:
- Add AnimatedText component for staggered letter animation
- Display 'UniClipboard' with fade-in-up effect (50ms per letter)
- Replace bounce dots with elegant pulse-wave animation
- Add accessibility attributes (role, aria-live)
- Maintain className prop for fade-out transition

Design: docs/plans/2026-01-18-loadingscreen-redesign.md

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 3: Manual Testing

**Files:**

- No files modified

**Step 1: Start development server**

Run: `bun tauri dev`
Expected: App launches with new LoadingScreen visible on startup

**Step 2: Verify animations**

1. **Letter animation**: Watch "UniClipboard" letters fade in sequentially
   - Expected: Each letter appears 50ms after the previous one
   - Expected: Letters move up slightly while fading in
   - Expected: Total animation duration ~1.15s

2. **Status text**: Wait for status text to appear
   - Expected: "正在初始化..." (or equivalent in your language) fades in after ~1.35s
   - Expected: Fade-in duration 300ms

3. **Pulse dots**: Observe the three dots
   - Expected: Dots animate in wave pattern (not bounce)
   - Expected: Scale: 0.8 → 1.2 → 0.8
   - Expected: Opacity: 0.4 → 1.0 → 0.4
   - Expected: Cycle duration 1.2s

**Step 3: Test theme switching**

1. Toggle between light and dark themes
   - Expected: Product name text color adapts correctly
   - Expected: Status text remains readable in both themes
   - Expected: Pulse dots use primary color in both themes

**Step 4: Test fade-out transition**

Wait for backend initialization to complete

- Expected: LoadingScreen fades out smoothly (300ms transition)
- Expected: No jarring jump or flicker

**Step 5: Check responsiveness**

Resize window to mobile width (320px)

- Expected: Product name scales down (text-4xl instead of text-5xl)
- Expected: All elements remain centered
- Expected: No horizontal overflow

**Step 6: Verify i18n**

Change language in settings (if available)

- Expected: Status text displays in selected language
- Expected: "UniClipboard" remains unchanged (brand name doesn't translate)

---

## Task 4: Verify Success Criteria

**Files:**

- No files modified

**Step 1: Run through checklist**

- [ ] Product name "UniClipboard" displays with letter-by-letter animation
- [ ] Status text fades in after product name animation completes
- [ ] Pulse dots animate in wave pattern with proper timing
- [ ] Design works in both light and dark themes
- [ ] Animations are smooth (60fps) on all platforms
- [ ] Component maintains existing props interface (`className`)
- [ ] i18n support preserved for status text

**Step 2: Performance check**

Open DevTools Performance tab:

1. Start recording
2. Refresh the app (triggers loading screen)
3. Stop recording after loading screen completes
4. Check for frame drops

Expected: Frames stay at or near 60fps during animations

**Step 3: Cross-platform check**

If you have access to other platforms:

- [ ] Windows: Verify animations work correctly
- [ ] Linux: Verify animations work correctly
- [ ] macOS: Already verified (primary development platform)

---

## Summary

**Files Modified:**

1. `src/styles/globals.css` - Added custom CSS animations
2. `src/components/LoadingScreen.tsx` - Complete redesign

**Lines of Code:**

- CSS: ~30 lines
- TypeScript: ~50 lines

**Estimated Time:** 15-20 minutes

**Testing Time:** 10 minutes

**Total Time:** 25-30 minutes

---

## Design Reference

For detailed design rationale and animation specifications, see:

- `docs/plans/2026-01-18-loadingscreen-redesign.md`

---

## Troubleshooting

**Issue: Animations don't play**

- Check browser console for CSS errors
- Verify `src/styles/globals.css` is imported in `src/main.tsx`
- Clear browser cache and restart dev server

**Issue: Letters appear all at once**

- Verify `animationDelay` is set with `style` attribute (not inline className)
- Check that `fade-in-up` animation is defined in globals.css

**Issue: Pulse dots bounce instead of wave**

- Verify `animate-pulse-wave` class is applied (not `animate-bounce`)
- Check that negative animation delays are using style attribute

**Issue: Text doesn't scale on mobile**

- Verify Tailwind responsive classes: `text-4xl md:text-5xl`
- Check viewport meta tag in index.html
