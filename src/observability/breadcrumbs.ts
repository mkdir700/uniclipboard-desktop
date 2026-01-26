import { Sentry, sentryEnabled } from '@/observability/sentry'

export type UserIntent =
  | 'copy_clipboard'
  | 'paste_clipboard'
  | 'open_settings'
  | 'pair_device'
  | 'delete_entry'
  | 'search_entries'
  | 'toggle_favorite'

export function captureUserIntent(intent: UserIntent, context?: Record<string, unknown>) {
  if (!sentryEnabled) {
    return
  }
  Sentry.addBreadcrumb({
    category: 'user_intent',
    message: intent,
    level: 'info',
    data: context,
  })
}

export function captureStateChange(state: string, from: string, to: string) {
  if (!sentryEnabled) {
    return
  }
  Sentry.addBreadcrumb({
    category: 'state_change',
    message: `${state}: ${from} -> ${to}`,
    level: 'info',
  })
}
