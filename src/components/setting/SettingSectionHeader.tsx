interface SettingSectionHeaderProps {
  title: string
}

export function SettingSectionHeader({ title }: SettingSectionHeaderProps) {
  return (
    <div className="flex items-center gap-4 mb-4 px-6 pt-6">
      <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">{title}</h3>
      <div className="h-px flex-1 bg-border/50" />
    </div>
  )
}
