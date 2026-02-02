import { useState } from "react";
import SelectionOverlay from "./components/Capture/Overlay";
import SettingsModal from "./components/Settings/SettingsModal";
import Dashboard from "./components/Capture/Dashboard";
import "./App.css";

function App() {
  // Determine mode based on URL path (set by Tauri when creating windows)
  const getInitialMode = () => {
    const path = window.location.pathname;
    if (path === "/overlay") return "overlay";
    if (path === "/settings") return "settings";
    return "dashboard";
  };

  const [mode] = useState<"dashboard" | "overlay" | "settings">(getInitialMode);

  // No event listeners here - each window type knows its role from the URL
  // This prevents cross-window event leakage

  if (mode === "overlay") {
    return <SelectionOverlay />;
  }

  if (mode === "settings") {
    return <SettingsModal />;
  }

  // Dashboard mode - main window background
  return (
    <div className="container">
      <Dashboard />
    </div>
  );
}

export default App;
