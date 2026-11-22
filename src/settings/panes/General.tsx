import { useEffect, useState } from "react";
import { ShortcutRecorder } from "../../components/ShortcutRecorder";
import { Row, Section, Segmented, Select, Toggle } from "../../components/ui";
import { api, type Mic } from "../../lib/ipc";
import { useStore } from "../store";
import { PermissionBanner } from "../Permissions";

const ACCENTS = ["#7B61FF", "#FF5A7A", "#FF8A3D", "#22B573", "#1E9BF0", "#8E8E93"];

export function General() {
  const { settings, status, update } = useStore();
  const [mics, setMics] = useState<Mic[]>([]);
  const [launch, setLaunch] = useState(false);

  useEffect(() => {
    api.listMics().then(setMics);
    api.getLaunchAtLogin().then(setLaunch);
  }, []);

  return (
    <>
      <PermissionBanner />

      <Section
        title="Shortcut"
        footer={
          status.platform === "macos" && settings.shortcut.toLowerCase() === "fn" ? (
            <>
              Using fn? Set <b>System Settings → Keyboard → Press 🌐 key to → Do Nothing</b>, so the emoji picker doesn't open.{" "}
              <a onClick={() => api.openPrivacySettings("keyboard")}>Open Keyboard settings</a>
            </>
          ) : status.platform === "linux" ? (
            <>
              On Wayland you can also bind <code>yaptap --toggle</code> to a shortcut in your desktop's settings.
            </>
          ) : undefined
        }
      >
        <Row label="Dictation key" hint="Works in any app.">
          <ShortcutRecorder />
        </Row>
        <Row
          label="How it works"
          hint={
            settings.triggerMode === "auto"
              ? "Hold to talk, or tap once to keep listening hands-free."
              : settings.triggerMode === "hold"
                ? "Recording stops when you let go."
                : "Press once to start, again to stop."
          }
        >
          <Segmented
            value={settings.triggerMode}
            onChange={(triggerMode) => update({ triggerMode })}
            options={[
              { value: "auto", label: "Hold or tap" },
              { value: "hold", label: "Hold" },
              { value: "toggle", label: "Toggle" },
            ]}
          />
        </Row>
      </Section>

      <Section title="Typing">
        <Row label="Insert text by" hint={settings.insertMethod === "type" ? "Slower, but works where paste is blocked." : undefined}>
          <Segmented
            value={settings.insertMethod}
            onChange={(insertMethod) => update({ insertMethod })}
            options={[
              { value: "paste", label: "Pasting" },
              { value: "type", label: "Typing" },
              { value: "copy", label: "Copy only" },
            ]}
          />
        </Row>
        {settings.insertMethod === "paste" && (
          <Row label="Restore clipboard after pasting">
            <Toggle checked={settings.restoreClipboard} onChange={(restoreClipboard) => update({ restoreClipboard })} />
          </Row>
        )}
        <Row label="Add a space after each dictation">
          <Toggle checked={settings.trailingSpace} onChange={(trailingSpace) => update({ trailingSpace })} />
        </Row>
      </Section>

      <Section title="Microphone & sound">
        <Row label="Microphone">
          <Select
            value={settings.microphone ?? ""}
            onChange={(v) => update({ microphone: v || null })}
            options={[
              { value: "", label: "System default" },
              ...mics.map((m) => ({ value: m.id, label: m.name })),
            ]}
          />
        </Row>
        <Row label="Sounds" hint="A soft tick when recording starts and stops.">
          <Toggle checked={settings.sounds} onChange={(sounds) => update({ sounds })} />
        </Row>
      </Section>

      <Section title="Appearance">
        <Row label="Pill position">
          <Segmented
            value={settings.overlayPosition}
            onChange={(overlayPosition) => update({ overlayPosition })}
            options={[
              { value: "bottom", label: "Bottom" },
              { value: "top", label: "Top" },
            ]}
          />
        </Row>
        <Row label="Accent color">
          <div className="swatches">
            {ACCENTS.map((c) => (
              <button
                key={c}
                className={`swatch ${settings.accent.toLowerCase() === c.toLowerCase() ? "is-active" : ""}`}
                style={{ background: c }}
                onClick={() => update({ accent: c })}
                aria-label={c}
              />
            ))}
          </div>
        </Row>
        <Row label="Open at login">
          <Toggle
            checked={launch}
            onChange={async (v) => {
              await api.setLaunchAtLogin(v);
              setLaunch(await api.getLaunchAtLogin());
            }}
          />
        </Row>
      </Section>
    </>
  );
}
