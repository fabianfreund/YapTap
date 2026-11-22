import type { Platform } from "./ipc";

const MAC: Record<string, string> = {
  Cmd: "⌘", CmdLeft: "Left ⌘", CmdRight: "Right ⌘",
  Opt: "⌥", OptLeft: "Left ⌥", OptRight: "Right ⌥",
  Ctrl: "⌃", CtrlLeft: "Left ⌃", CtrlRight: "Right ⌃",
  Shift: "⇧", ShiftLeft: "Left ⇧", ShiftRight: "Right ⇧",
  Fn: "fn", Space: "Space", Escape: "Esc", Return: "↩", Tab: "⇥",
};

const PC: Record<string, string> = {
  Cmd: "Win", CmdLeft: "Left Win", CmdRight: "Right Win",
  Opt: "Alt", OptLeft: "Left Alt", OptRight: "Right Alt",
  Ctrl: "Ctrl", CtrlLeft: "Left Ctrl", CtrlRight: "Right Ctrl",
  Shift: "Shift", ShiftLeft: "Left Shift", ShiftRight: "Right Shift",
  Escape: "Esc",
};

/** Turns handy-keys notation ("Ctrl+Win", "OptRight") into key caps. */
export function shortcutKeys(shortcut: string, platform: Platform): string[] {
  const names = platform === "macos" ? MAC : PC;
  return shortcut
    .split("+")
    .map((p) => p.trim())
    .filter(Boolean)
    .map((p) => {
      const key = Object.keys(names).find((k) => k.toLowerCase() === p.toLowerCase().replace(/_/g, ""));
      if (key) return names[key];
      if (/^win(dows)?$|^super$|^meta$/i.test(p)) return platform === "macos" ? "⌘" : "Win";
      if (/^(alt|option)$/i.test(p)) return platform === "macos" ? "⌥" : "Alt";
      if (/^control$/i.test(p)) return platform === "macos" ? "⌃" : "Ctrl";
      return p.length === 1 ? p.toUpperCase() : p;
    });
}

export function bytes(n: number): string {
  if (n >= 1e9) return `${(n / 1e9).toFixed(1)} GB`;
  if (n >= 1e6) return `${Math.round(n / 1e6)} MB`;
  return `${Math.round(n / 1e3)} KB`;
}

export function relativeDay(iso: string): string {
  const d = new Date(iso);
  const today = new Date();
  const yesterday = new Date();
  yesterday.setDate(today.getDate() - 1);
  if (d.toDateString() === today.toDateString()) return "Today";
  if (d.toDateString() === yesterday.toDateString()) return "Yesterday";
  return d.toLocaleDateString(undefined, { weekday: "long", month: "short", day: "numeric" });
}

export function time(iso: string): string {
  return new Date(iso).toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
}

/** Languages offered for "lock language". Parakeet v3 covers the first 25. */
export const LANGUAGES: [string, string][] = [
  ["en", "English"], ["de", "German"], ["fr", "French"], ["es", "Spanish"], ["it", "Italian"],
  ["pt", "Portuguese"], ["nl", "Dutch"], ["pl", "Polish"], ["sv", "Swedish"], ["da", "Danish"],
  ["fi", "Finnish"], ["cs", "Czech"], ["sk", "Slovak"], ["sl", "Slovenian"], ["hr", "Croatian"],
  ["hu", "Hungarian"], ["ro", "Romanian"], ["bg", "Bulgarian"], ["el", "Greek"], ["et", "Estonian"],
  ["lv", "Latvian"], ["lt", "Lithuanian"], ["mt", "Maltese"], ["ru", "Russian"], ["uk", "Ukrainian"],
  ["ja", "Japanese"], ["zh", "Chinese"], ["ko", "Korean"], ["tr", "Turkish"], ["ar", "Arabic"],
  ["hi", "Hindi"],
];
