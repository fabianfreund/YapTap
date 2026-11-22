import { useState } from "react";
import { Onboarding } from "./Onboarding";
import { Dictionary } from "./panes/Dictionary";
import { General } from "./panes/General";
import { History } from "./panes/History";
import { Models } from "./panes/Models";
import { Privacy } from "./panes/Privacy";
import { StoreProvider, useStore } from "./store";
import { useMissing } from "./Permissions";
import "./app.css";

const PANES = [
  { id: "general", label: "General", icon: "M8 2.5v1.2m0 8.6v1.2M2.5 8h1.2m8.6 0h1.2M4.1 4.1l.9.9m6 6l.9.9m0-7.8l-.9.9m-6 6l-.9.9M8 10.2a2.2 2.2 0 100-4.4 2.2 2.2 0 000 4.4z", Pane: General },
  { id: "models", label: "Models", icon: "M8 2l5.5 3v6L8 14l-5.5-3V5L8 2zm0 0v12M2.5 5L8 8l5.5-3", Pane: Models },
  { id: "dictionary", label: "Dictionary", icon: "M3.5 2.5h7a2 2 0 012 2v9h-7a2 2 0 01-2-2v-9zm0 9a2 2 0 012-2h7", Pane: Dictionary },
  { id: "history", label: "History", icon: "M8 4.5V8l2.5 1.5M8 13.5a5.5 5.5 0 110-11 5.5 5.5 0 010 11z", Pane: History },
  { id: "privacy", label: "Privacy & About", icon: "M8 2l5 2v4c0 3-2.2 5.2-5 6-2.8-.8-5-3-5-6V4l5-2z", Pane: Privacy },
] as const;

export function SettingsApp() {
  return (
    <StoreProvider>
      <Shell />
    </StoreProvider>
  );
}

function Shell() {
  const { settings, status } = useStore();
  const [pane, setPane] = useState<(typeof PANES)[number]["id"]>("general");
  const missing = useMissing();
  if (!settings.onboarded) return <Onboarding />;
  const { Pane, label } = PANES.find((p) => p.id === pane)!;
  const ready = missing.length === 0 && status.shortcutReady && status.engine.status !== "error";

  return (
    <div className={`shell shell--${status.platform}`}>
      <aside className="sidebar" data-tauri-drag-region>
        <div className="brand" data-tauri-drag-region>
          <img src="/logo.svg" alt="" />
          <div>
            <div className="brand-name">YapTap</div>
            <div className={`brand-status ${ready ? "ok" : ""}`}>
              <i />
              {ready
                ? status.engine.status === "loading"
                  ? "Loading model…"
                  : "Ready"
                : "Setup needed"}
            </div>
          </div>
        </div>
        <nav>
          {PANES.map((p) => (
            <button key={p.id} className={p.id === pane ? "is-active" : ""} onClick={() => setPane(p.id)}>
              <svg viewBox="0 0 16 16"><path d={p.icon} /></svg>
              {p.label}
            </button>
          ))}
        </nav>
        <div className="sidebar-foot">Local · Private · Free</div>
      </aside>
      <main className="content">
        <header className="content-head" data-tauri-drag-region>
          <h2>{label}</h2>
        </header>
        <div className="content-body">
          <Pane />
        </div>
      </main>
    </div>
  );
}
