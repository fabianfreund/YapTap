// Typed bridge to the Rust side. Every command and event lives here.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Platform = "macos" | "windows" | "linux";
export type TriggerMode = "auto" | "hold" | "toggle";
export type InsertMethod = "paste" | "type" | "copy";
export type OverlayPosition = "bottom" | "top";
export type HistoryRetention = "forever" | "week" | "day" | "never";

export interface Replacement {
  from: string;
  to: string;
}

export interface Settings {
  onboarded: boolean;
  shortcut: string;
  triggerMode: TriggerMode;
  model: string;
  language: string | null;
  microphone: string | null;
  insertMethod: InsertMethod;
  restoreClipboard: boolean;
  trailingSpace: boolean;
  sounds: boolean;
  overlayPosition: OverlayPosition;
  accent: string;
  vocabulary: string[];
  replacements: Replacement[];
  historyRetention: HistoryRetention;
  offline: boolean;
}

export interface Status {
  platform: Platform;
  version: string;
  engine: { status: "none" | "loading" | "ready" | "error"; model: string | null; error: string | null };
  shortcutReady: boolean;
  shortcutError: string | null;
  microphone: "granted" | "denied" | "notDetermined" | "unknown";
  accessibility: boolean | null;
}

export interface Model {
  id: string;
  name: string;
  description: string;
  kind: "speech" | "vad";
  languages: number;
  speed: number;
  accuracy: number;
  license: string;
  filename: string;
  size: number;
  installed: boolean;
  downloading: number | null;
}

export interface Mic {
  id: string;
  name: string;
  isDefault: boolean;
}

export interface HistoryEntry {
  id: number;
  at: string;
  raw: string;
  text: string;
  model: string;
  durationMs: number;
  inserted: boolean;
}

export type PillStateName = "listening" | "locked" | "processing" | "done" | "error" | "hidden";
export interface PillState {
  state: PillStateName;
  message: string | null;
}

export interface Progress {
  id: string;
  downloaded: number;
  total: number;
}

export const api = {
  getSettings: () => invoke<Settings>("get_settings"),
  setSettings: (settings: Settings) => invoke<Settings>("set_settings", { settings }),
  getStatus: () => invoke<Status>("get_status"),
  listModels: () => invoke<Model[]>("list_models"),
  downloadModel: (id: string) => invoke<void>("download_model", { id }),
  cancelDownload: (id: string) => invoke<void>("cancel_download", { id }),
  deleteModel: (id: string) => invoke<void>("delete_model", { id }),
  listMics: () => invoke<Mic[]>("list_mics"),
  getHistory: () => invoke<HistoryEntry[]>("get_history"),
  deleteHistory: (id: number) => invoke<void>("delete_history", { id }),
  clearHistory: () => invoke<void>("clear_history"),
  copyText: (text: string) => invoke<void>("copy_text", { text }),
  toggleDictation: () => invoke<void>("toggle_dictation"),
  cancelDictation: () => invoke<void>("cancel_dictation"),
  recordShortcut: () => invoke<string>("record_shortcut"),
  requestMicrophone: () => invoke<void>("request_microphone"),
  openPrivacySettings: (kind: "microphone" | "accessibility" | "keyboard") =>
    invoke<void>("open_privacy_settings", { kind }),
  getLaunchAtLogin: () => invoke<boolean>("get_launch_at_login"),
  setLaunchAtLogin: (enabled: boolean) => invoke<void>("set_launch_at_login", { enabled }),
  dataFolder: () => invoke<string>("data_folder"),
};

type Events = {
  pill: PillState;
  level: number;
  status: Status;
  "download-progress": Progress;
  "models-changed": null;
  "history-changed": null;
  "settings-changed": Settings;
};

export function on<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): Promise<UnlistenFn> {
  return listen<Events[K]>(event, (e) => handler(e.payload));
}
