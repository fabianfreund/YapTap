import { useState } from "react";
import { Button, Section } from "../../components/ui";
import { useStore } from "../store";

export function Dictionary() {
  const { settings, update } = useStore();
  const [word, setWord] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");

  const addWord = () => {
    const w = word.trim();
    if (!w || settings.vocabulary.includes(w)) return;
    update({ vocabulary: [...settings.vocabulary, w] });
    setWord("");
  };
  const addReplacement = () => {
    if (!from.trim()) return;
    update({ replacements: [...settings.replacements, { from: from.trim(), to }] });
    setFrom("");
    setTo("");
  };

  return (
    <>
      <Section
        title="Words"
        footer="Names, brands and jargon the model should know. Only models that support custom words use this list. If a word still comes out wrong, add a replacement below."
      >
        <form
          className="inline-form"
          onSubmit={(e) => {
            e.preventDefault();
            addWord();
          }}
        >
          <input className="input" placeholder="e.g. YapTap, Kubernetes, Anneliese" value={word} onChange={(e) => setWord(e.target.value)} />
          <Button type="submit" variant="primary" disabled={!word.trim()}>
            Add
          </Button>
        </form>
        {settings.vocabulary.length > 0 && (
          <div className="chips">
            {settings.vocabulary.map((w) => (
              <span className="chip" key={w}>
                {w}
                <button
                  aria-label={`Remove ${w}`}
                  onClick={() => update({ vocabulary: settings.vocabulary.filter((x) => x !== w) })}
                >
                  ×
                </button>
              </span>
            ))}
          </div>
        )}
      </Section>

      <Section title="Replacements" footer="When YapTap hears the left side, it writes the right side. Whole words, any capitalization.">
        <form
          className="inline-form"
          onSubmit={(e) => {
            e.preventDefault();
            addReplacement();
          }}
        >
          <input className="input" placeholder="Heard" value={from} onChange={(e) => setFrom(e.target.value)} />
          <span className="arrow">→</span>
          <input className="input" placeholder="Write" value={to} onChange={(e) => setTo(e.target.value)} />
          <Button type="submit" variant="primary" disabled={!from.trim()}>
            Add
          </Button>
        </form>
        {settings.replacements.map((r, i) => (
          <div className="row" key={i}>
            <div className="row-text">
              <div className="row-label">
                <span className="muted">{r.from}</span> <span className="arrow">→</span> {r.to || <i className="muted">(nothing)</i>}
              </div>
            </div>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => update({ replacements: settings.replacements.filter((_, j) => j !== i) })}
            >
              Remove
            </Button>
          </div>
        ))}
      </Section>

      <Section title="Voice commands">
        <div className="row">
          <div className="row-text">
            <div className="row-label">"new line" · "new paragraph"</div>
            <div className="row-hint">Also in German: "neue Zeile" · "neuer Absatz"</div>
          </div>
        </div>
      </Section>
    </>
  );
}
