import { useMemo } from "react";
import { ModelCard } from "../../components/ModelCard";
import { Row, Section, Select } from "../../components/ui";
import { LANGUAGES } from "../../lib/format";
import { useStore } from "../store";

export function Models() {
  const { settings, models, update } = useStore();
  const speech = models.filter((m) => m.kind === "speech");
  const current = models.find((m) => m.id === settings.model);
  const languages = useMemo(
    () => (current?.languages === 25 ? LANGUAGES.slice(0, 25) : LANGUAGES),
    [current?.languages],
  );

  return (
    <>
      <Section
        title="Language"
        footer={
          settings.model === "parakeet-v3"
            ? "Parakeet detects the language by itself and can't be locked. To always get one language, use a Whisper model."
            : "Locking the language makes Whisper faster and avoids mix-ups."
        }
      >
        <Row label="Spoken language">
          <Select
            value={settings.language ?? ""}
            onChange={(v) => update({ language: v || null })}
            options={[{ value: "", label: "Detect automatically" }, ...languages.map(([value, label]) => ({ value, label }))]}
          />
        </Row>
      </Section>

      <Section title="Speech models" footer="Models run on your computer. They're downloaded once from Hugging Face and checked before use.">
        <div className="models">
          {speech.map((m) => (
            <ModelCard key={m.id} model={m} />
          ))}
        </div>
      </Section>
    </>
  );
}
