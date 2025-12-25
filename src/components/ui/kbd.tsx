interface KbdProps extends React.HTMLAttributes<HTMLElement> {
  children: React.ReactNode;
}

const Kbd = ({ children, className, ...props }: KbdProps) => {
  return (
    <kbd
      className="inline-flex h-5 min-w-5 items-center justify-center rounded border border-border/40 bg-muted px-1.5 text-[0.625rem] font-medium text-muted-foreground shadow-sm"
      {...props}
    >
      {children}
    </kbd>
  );
};

export { Kbd };
