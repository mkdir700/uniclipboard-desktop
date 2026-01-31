import { Loader2 } from 'lucide-react'
import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'
import { Sentry } from '@/observability/sentry'

interface FeedbackDialogProps {
  children: React.ReactNode
}

export function FeedbackDialog({ children }: FeedbackDialogProps) {
  const { t } = useTranslation()
  const [open, setOpen] = React.useState(false)
  const [content, setContent] = React.useState('')
  const [email, setEmail] = React.useState('')
  const [isSubmitting, setIsSubmitting] = React.useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!content.trim()) return

    setIsSubmitting(true)
    try {
      const eventId = Sentry.captureMessage('User Feedback')
      Sentry.captureFeedback({
        message: content,
        name: 'User',
        email: email || undefined,
        associatedEventId: eventId,
      })

      toast.success(t('feedback.modal.success'))
      setOpen(false)
      setContent('')
      setEmail('')
    } catch (error) {
      console.error('Failed to send feedback:', error)
      toast.error(t('feedback.modal.error'))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>{children}</DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>{t('feedback.modal.title')}</DialogTitle>
          <DialogDescription>{t('feedback.modal.description')}</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="grid gap-4 py-4">
          <div className="grid gap-2">
            <label
              htmlFor="feedback-content"
              className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
            >
              {t('feedback.modal.label.content')}
            </label>
            <textarea
              id="feedback-content"
              className={cn(
                'flex min-h-[100px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50'
              )}
              placeholder={t('feedback.modal.placeholder.content')}
              value={content}
              onChange={e => setContent(e.target.value)}
              required
            />
          </div>
          <div className="grid gap-2">
            <label
              htmlFor="feedback-email"
              className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
            >
              {t('feedback.modal.label.email')}
            </label>
            <Input
              id="feedback-email"
              type="email"
              placeholder={t('feedback.modal.placeholder.email')}
              value={email}
              onChange={e => setEmail(e.target.value)}
            />
          </div>
          <DialogFooter>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t('feedback.modal.submit')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
