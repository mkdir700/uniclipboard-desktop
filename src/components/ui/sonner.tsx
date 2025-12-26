import { useSetting } from "@/contexts/SettingContext"
import { toast } from "sonner"

type ToasterProps = React.ComponentProps<typeof toast.Toaster>

function Toaster({ ...props }: ToasterProps) {
  const { setting } = useSetting()
  const theme = setting?.general.theme || "system"

  return (
    <toast.Toaster
      theme={theme as ToasterProps["theme"]}
      className="toaster group"
      toastOptions={{
        classNames: {
          toast:
            "group toast group-[.toaster]:bg-background group-[.toaster]:text-foreground group-[.toaster]:border-border group-[.toaster]:shadow-lg",
          description: "group-[.toast]:text-muted-foreground",
          actionButton:
            "group-[.toast]:bg-primary group-[.toast]:text-primary-foreground",
          cancelButton:
            "group-[.toast]:bg-muted group-[.toast]:text-muted-foreground",
        },
      }}
      {...props}
    />
  )
}

export { Toaster, toast }
