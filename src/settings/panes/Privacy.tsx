import { useEffect, useState } from "react";
import { Row, Section, Segmented, Toggle } from "../../components/ui";
import { api } from "../../lib/ipc";
import { useStore } from "../store";

export function Privacy() {
  const { settings, status, update } = useStore();
  const [folder, setFolder] = useState("");
  useEffect(() => void api.dataFolder().then(setFolder), []);

  return (
    <>
      <div className="promise">
        <div className="promise-icon">🔒</div>
        <div>
          <b>Everything stays on this computer.</b>
          <p>
            Your voice is turned into text by a model running locally. YapTap has no cloud, no account and no analytics.
            The only network requests are model downloads from Hugging Face.
          </p>
        </div>
      </div>

      <Section title="Privacy">
        <Row label="Offline mode" hint="Blocks every network request, including model downloads.">
          <Toggle checked={settings.offline} onChange={(offline) => update({ offline })} />
        </Row>
        <Row label="Keep history">
          <Segmented
            value={settings.historyRetention}
            onChange={(historyRetention) => update({ historyRetention })}
            options={[
              { value: "forever", label: "Forever" },
              { value: "week", label: "7 days" },
              { value: "day", label: "24 hours" },
              { value: "never", label: "Never" },
            ]}
          />
        </Row>
        <Row label="Recordings" hint="Audio is never saved. It's kept in memory until the text is ready, then dropped." />
        <Row label="Data folder" hint={<code className="path">{folder}</code>} />
      </Section>

      <Section title="About">
        <Row label="YapTap" hint={`Version ${status.version} · ${status.platform}`} />
        <Row
          label="Open source"
          hint={
            <>
              MIT license. Made by Fabian Builds, part of the Appetizer collection. Built with Tauri, transcribe.cpp and parts
              of Handy (MIT).
            </>
          }
        />
        <Row
          label="Model credits"
          hint="Parakeet TDT 0.6B v3 by NVIDIA (CC-BY-4.0). Whisper by OpenAI (MIT). Silero VAD (MIT). GGUF builds by the transcribe.cpp project."
        />
      </Section>
    </>
  );
}
