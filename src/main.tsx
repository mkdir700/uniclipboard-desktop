import React from "react";
import ReactDOM from "react-dom/client";
import { Provider } from "react-redux";
import { store } from "./store";
import App from "./App";
import "./i18n";
import { initLogger } from "./utils/logger";

const applyPlatformTypographyScale = () => {
  if (typeof navigator === "undefined" || typeof document === "undefined") {
    return;
  }

  const ua = navigator.userAgent || "";
  const isWindows = ua.includes("Windows");

  if (!isWindows) {
    return;
  }

  const root = document.documentElement;

  root.style.setProperty("--font-size-sm", "0.75rem");
  root.style.setProperty("--font-size-base", "0.875rem");
  root.style.setProperty("--font-size-lg", "1rem");
  root.style.setProperty("--line-height-ui", "1.35");
};

applyPlatformTypographyScale();
void initLogger();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Provider store={store}>
      <App />
    </Provider>
  </React.StrictMode>,
);
