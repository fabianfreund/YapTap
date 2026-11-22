import { useState } from "react";
import { ModelCard } from "../components/ModelCard";
import { ShortcutRecorder } from "../components/ShortcutRecorder";
import { Button } from "../components/ui";
import { api } from "../lib/ipc";
import { shortcutKeys } from "../lib/format";
import { useStore } from "./store";

type Step = "welcome" | "mic" | "access" | "model" | "try";

/** First-run setup: permissions, model download, and a practice field. */
export function Onboarding() {
  const { status, settings, models, update } = useStore();
  const steps: Step[] = ["welcome", "mic", ...(status.platform === "macos" ? (["access"] as Step[]) : []), "model", "try"];
  const [i, setI] = useState(0);
  const step = steps[i];
  const next = () => setI((n) => Math.min(n + 1, steps.length - 1));
  const back = () => setI((n) => Math.max(n - 1, 0));
  const model = models.find((m) => m.id === settings.model);
  const keys = shortcutKeys(settings.shortcut, status.platform);

  return (
    <div className="onboarding" data-tauri-drag-region>
      <div className="onboarding-card">
        <div className="steps">
          {steps.map((s, n) => (
            <span key={s} className={n <= i ? "on" : ""} />
          ))}
        </div>

        {step === "welcome" && (
          <div className="ob-step">
            <img className="ob-logo" src="/logo.svg" alt="" />
            <h1>Hold a key, talk, let go.</h1>
            <p>
              YapTap turns your voice into text wherever your cursor is. It runs entirely on this computer: no cloud, no
              account, no limits.
            </p>
            <Button variant="primary" size="lg" onClick={next}>
              Get started
            </Button>
          </div>
        )}

        {step === "mic" && (
          <div className="ob-step">
            <div className="ob-icon">🎙️</div>
            <h1>Let YapTap hear you</h1>
            <p>The microphone is only on while you hold the key. Audio is never saved or sent anywhere.</p>
            {status.microphone === "granted" || status.microphone === "unknown" ? (
              <>
                <div className="ob-ok">✓ Microphone ready</div>
                <Button variant="primary" size="lg" onClick={next}>
                  Continue
                </Button>
              </>
            ) : status.microphone === "denied" ? (
              <>
                <p className="muted">Access was denied. Turn on YapTap in Privacy settings, then come back.</p>
                <Button variant="primary" size="lg" onClick={() => api.openPrivacySettings("microphone")}>
                  Open Settings
                </Button>
              </>
            ) : (
              <Button variant="primary" size="lg" onClick={() => api.requestMicrophone().catch(() => {})}>
                Allow microphone
              </Button>
            )}
          </div>
        )}

        {step === "access" && (
          <div className="ob-step">
            <div className="ob-icon">⌨️</div>
            <h1>Let YapTap type for you</h1>
            <p>
              macOS asks for <b>Accessibility</b> access so YapTap can notice your shortcut and paste the text into the app
              you're using.
            </p>
            {status.accessibility ? (
              <>
                <div className="ob-ok">✓ Access granted</div>
                <Button variant="primary" size="lg" onClick={next}>
                  Continue
                </Button>
              </>
            ) : (
              <>
                <Button variant="primary" size="lg" onClick={() => api.openPrivacySettings("accessibility")}>
                  Open Accessibility settings
                </Button>
                <p className="muted small">Turn on YapTap in the list. This page updates by itself.</p>
              </>
            )}
          </div>
        )}

        {step === "model" && (
          <div className="ob-step ob-step--wide">
            <h1>Download the speech model</h1>
            <p>A one-time download. After that, YapTap works offline.</p>
            <div className="models">
              {models
                .filter((m) => m.kind === "speech")
                .map((m) => (
                  <ModelCard key={m.id} model={m} compact />
                ))}
            </div>
            <Button variant="primary" size="lg" disabled={!model?.installed} onClick={next}>
              {model?.installed ? "Continue" : "Waiting for download…"}
            </Button>
          </div>
        )}

        {step === "try" && (
          <div className="ob-step ob-step--wide">
            <h1>Try it</h1>
            <p>
              Click into the box, then <b>hold {keys.join(" + ")}</b> and say something. Let go when you're done.
              {settings.triggerMode === "auto" && " Or tap it once to talk hands-free, and tap again to finish."}
            </p>
            <div className="ob-shortcut">
              <span className="muted">Shortcut</span>
              <ShortcutRecorder />
            </div>
            {status.platform === "macos" && settings.shortcut.toLowerCase() === "fn" && (
              <p className="muted small">
                Tip: if the emoji picker opens, set Keyboard → "Press 🌐 key to" → Do Nothing.{" "}
                <a onClick={() => api.openPrivacySettings("keyboard")}>Open Keyboard settings</a>
              </p>
            )}
            <textarea className="input practice" placeholder="Your words will appear here…" />
            {!status.shortcutReady && status.shortcutError && <div className="field-error">{status.shortcutError}</div>}
            <Button variant="primary" size="lg" onClick={() => update({ onboarded: true })}>
              Finish
            </Button>
          </div>
        )}

        {i > 0 && (
          <button className="ob-back" onClick={back}>
            ← Back
          </button>
        )}
      </div>
    </div>
  );
}
