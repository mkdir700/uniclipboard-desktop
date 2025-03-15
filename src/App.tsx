import DashboardPage from "./pages/DashboardPage";
import HistoryPage from "./pages/HistoryPage";
import DevicesPage from "./pages/DevicesPage";
import SettingsPage from "./pages/SettingsPage";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { SettingProvider } from "./contexts/SettingContext";
import "./App.css";

function App() {
  return (
    <SettingProvider>
      <Router>
        <Routes>
          <Route path="/" element={<DashboardPage />} />
          <Route path="/history" element={<HistoryPage />} />
          <Route path="/devices" element={<DevicesPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Routes>
      </Router>
    </SettingProvider>
  );
}

export default App;
