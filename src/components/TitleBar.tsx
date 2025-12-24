
interface TitleBarProps {
  className?: string;
}

export const TitleBar = ({ className }: TitleBarProps) => {
  return (
    <div
      data-tauri-drag-region
      className={`fixed top-0 left-0 right-0 h-8 z-9999 bg-transparent select-none cursor-default ${
        className || ""
      }`}
    />
  );
};

export default TitleBar;
