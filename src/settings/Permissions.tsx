import { Button } from "../components/ui";
import { api } from "../lib/ipc";
import { useStore } from "./store";

/** What's missing before dictation can work, with one-click fixes. */
export function useMissing() {
  const { status, settings, models } = useStore();
  const missing: { key: string; title: string; hint: string; action: () => void; label: string }[] = [];

  if (status.microphone === "denied" || status.microphone === "notDetermined") {
    missing.push({
      key: "mic",
      title: "Microphone access",
      hint: "YapTap needs the microphone to hear you. Audio never leaves this computer.",
      action: () =>
        status.microphone === "denied" ? api.openPrivacySettings("microphone") : api.requestMicrophone(),
      label: status.microphone === "denied" ? "Open Settings" : "Allow",
    });
  }
  if (status.accessibility === false) {
    missing.push({
      key: "ax",
      title: "Accessibility access",
      hint: "Needed for the shortcut and to paste text into other apps.",
      action: () => api.openPrivacySettings("accessibility"),
      label: "Open Settings",
    });
  }
  const model = models.find((m) => m.id === settings.model);
  if (model && !model.installed) {
    missing.push({
      key: "model",
      title: "Speech model",
      hint: "Download the speech model once. After that, everything works offline.",
      action: () => api.downloadModel(model.id),
      label: "Download",
    });
  }
  return missing;
}

export function PermissionBanner() {
  const missing = useMissing();
  const { status } = useStore();
  if (missing.length === 0 && status.shortcutReady) return null;
  return (
    <div className="banner">
      <div className="banner-title">Almost ready</div>
      {missing.map((m) => (
        <div className="banner-row" key={m.key}>
          <div>
            <b>{m.title}</b>
            <div className="row-hint">{m.hint}</div>
          </div>
          <Button size="sm" variant="primary" onClick={m.action}>
            {m.label}
          </Button>
        </div>
      ))}
      {missing.length === 0 && !status.shortcutReady && status.shortcutError && (
        <div className="banner-row">
          <div>
            <b>Shortcut not active</b>
            <div className="row-hint">{status.shortcutError}</div>
          </div>
        </div>
      )}
    </div>
  );
}
