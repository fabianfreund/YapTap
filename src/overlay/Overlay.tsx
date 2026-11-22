import { useEffect, useRef, useState } from "react";
import { LevelBars } from "../components/LevelBars";
import { api, on, type PillState } from "../lib/ipc";
import "./overlay.css";

/** The floating pill shown while you dictate. */
export function Overlay() {
  const [pill, setPill] = useState<PillState>({ state: "hidden", message: null });
  const [seconds, setSeconds] = useState(0);
  const levelRef = useRef(0);

  useEffect(() => {
    const subs = [
      on("pill", (p) => {
        setPill(p);
        if (p.state === "listening") setSeconds(0);
        if (p.state !== "listening" && p.state !== "locked") levelRef.current = 0;
      }),
      on("level", (l) => {
        levelRef.current = l;
      }),
      on("settings-changed", (s) => applyAccent(s.accent)),
    ];
    api.getSettings().then((s) => applyAccent(s.accent));
    return () => subs.forEach((p) => p.then((off) => off()));
  }, []);

  const recording = pill.state === "listening" || pill.state === "locked";

  useEffect(() => {
    if (!recording) return;
    const id = setInterval(() => setSeconds((s) => s + 1), 1000);
    return () => clearInterval(id);
  }, [recording]);

  return (
    <div className="stage">
      <div className={`pill pill--${pill.state}`} role="status" aria-live="polite">
        {recording && (
          <>
            <button className="pill-btn" onClick={() => api.cancelDictation()} title="Cancel (Esc)" aria-label="Cancel">
              <svg viewBox="0 0 16 16"><path d="M4.5 4.5l7 7m0-7l-7 7" /></svg>
            </button>
            <LevelBars levelRef={levelRef} active={recording} />
            <span className="pill-time">
              {pill.state === "locked" && <i className="lock-dot" title="Hands-free" />}
              {Math.floor(seconds / 60)}:{String(seconds % 60).padStart(2, "0")}
            </span>
            <button className="pill-btn pill-btn--go" onClick={() => api.toggleDictation()} title="Done" aria-label="Done">
              <svg viewBox="0 0 16 16"><path d="M3.5 8.5l3 3 6-7" /></svg>
            </button>
          </>
        )}
        {pill.state === "processing" && (
          <div className="dots" aria-label="Transcribing">
            <span /><span /><span />
          </div>
        )}
        {pill.state === "done" && (
          <div className="pill-msg">
            <svg className="ok" viewBox="0 0 16 16"><path d="M3.5 8.5l3 3 6-7" /></svg>
            {pill.message && <span>{pill.message}</span>}
          </div>
        )}
        {pill.state === "error" && (
          <div className="pill-msg">
            <svg className="warn" viewBox="0 0 16 16"><path d="M8 4.5v4.5M8 11.6v.1" /></svg>
            <span>{pill.message}</span>
          </div>
        )}
      </div>
    </div>
  );
}

function applyAccent(color: string) {
  document.documentElement.style.setProperty("--accent", color);
}
