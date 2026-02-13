import type { ReactNode } from 'react'
import { SettingSectionHeader } from './SettingSectionHeader'
import { Card, CardContent } from '@/components/ui/card'

interface SettingCardProps {
  title: string
  children: ReactNode
  className?: string
}

export function SettingCard({ title, children, className }: SettingCardProps) {
  return (
    <Card className={className}>
      <SettingSectionHeader title={title} />
      <CardContent className="pt-0">{children}</CardContent>
    </Card>
  )
}
