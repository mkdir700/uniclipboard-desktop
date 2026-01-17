import { Outlet } from 'react-router-dom'

/**
 * Layout for the full-screen settings page.
 * This layout does not include the main sidebar and has a simpler structure.
 */
const SettingsFullLayout = () => {
  return (
    <div className="h-screen w-full flex flex-col bg-background text-foreground transition-colors duration-200">
      <Outlet />
    </div>
  )
}

export default SettingsFullLayout
