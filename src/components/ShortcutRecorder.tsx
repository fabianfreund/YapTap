import { useState } from "react";
import { api } from "../lib/ipc";
import { shortcutKeys } from "../lib/format";
import { useStore } from "../settings/store";
import { Button, Keycaps } from "./ui";

/** Shows the dictation shortcut and records a new one. */
export function ShortcutRecorder() {
  const { settings, status, update } = useStore();
  const [recording, setRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const record = async () => {
    setError(null);
    setRecording(true);
    try {
      const shortcut = await api.recordShortcut();
      setError(await update({ shortcut }));
    } catch (e) {
      if (!String(e).includes("cancelled")) setError(String(e));
    } finally {
      setRecording(false);
    }
  };

  return (
    <div className="shortcut">
      <div className="shortcut-row">
        {recording ? (
          <span className="shortcut-listening">Press keys…</span>
        ) : (
          <Keycaps keys={shortcutKeys(settings.shortcut, status.platform)} />
        )}
        <Button size="sm" onClick={record} disabled={recording}>
          {recording ? "Listening" : "Change"}
        </Button>
      </div>
      {error && <div className="field-error">{error}</div>}
    </div>
  );
}
