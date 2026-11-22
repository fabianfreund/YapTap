// Shared state for the settings window: settings, status, models.
import { createContext, useCallback, useContext, useEffect, useState, type ReactNode } from "react";
import { api, on, type Model, type Settings, type Status } from "../lib/ipc";

interface Store {
  settings: Settings;
  status: Status;
  models: Model[];
  progress: Record<string, number>;
  update: (patch: Partial<Settings>) => Promise<string | null>;
  refreshModels: () => void;
  refreshStatus: () => void;
}

const Ctx = createContext<Store | null>(null);

export function useStore(): Store {
  const s = useContext(Ctx);
  if (!s) throw new Error("useStore outside StoreProvider");
  return s;
}

export function StoreProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [status, setStatus] = useState<Status | null>(null);
  const [models, setModels] = useState<Model[]>([]);
  const [progress, setProgress] = useState<Record<string, number>>({});

  const refreshModels = useCallback(() => void api.listModels().then(setModels), []);
  const refreshStatus = useCallback(() => void api.getStatus().then(setStatus), []);

  useEffect(() => {
    api.getSettings().then(setSettings);
    refreshStatus();
    refreshModels();
    const subs = [
      on("status", setStatus),
      on("settings-changed", setSettings),
      on("models-changed", () => {
        refreshModels();
        setProgress({});
      }),
      on("download-progress", (p) => setProgress((old) => ({ ...old, [p.id]: p.downloaded / p.total }))),
    ];
    // Permissions change outside the app, so check again now and then.
    const poll = setInterval(refreshStatus, 2000);
    return () => {
      clearInterval(poll);
      subs.forEach((p) => p.then((off) => off()));
    };
  }, [refreshModels, refreshStatus]);

  useEffect(() => {
    if (settings) document.documentElement.style.setProperty("--accent", settings.accent);
  }, [settings]);

  const update = useCallback(
    async (patch: Partial<Settings>) => {
      if (!settings) return null;
      try {
        setSettings(await api.setSettings({ ...settings, ...patch }));
        return null;
      } catch (e) {
        return String(e);
      }
    },
    [settings],
  );

  if (!settings || !status) return null;
  return (
    <Ctx.Provider value={{ settings, status, models, progress, update, refreshModels, refreshStatus }}>
      {children}
    </Ctx.Provider>
  );
}
