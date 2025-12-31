import { Outlet } from 'react-router-dom'

/**
 * Layout for the standalone settings window.
 * This window does not include the main sidebar and has a simpler structure.
 */
const SettingsWindowLayout = () => {
  return (
    <div className="h-screen w-full flex flex-col bg-background text-foreground transition-colors duration-200">
      <Outlet />
    </div>
  )
}

export default SettingsWindowLayout
