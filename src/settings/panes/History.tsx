import { useEffect, useMemo, useState } from "react";
import { Button } from "../../components/ui";
import { api, on, type HistoryEntry } from "../../lib/ipc";
import { relativeDay, time } from "../../lib/format";

export function History() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [query, setQuery] = useState("");
  const [copied, setCopied] = useState<number | null>(null);

  useEffect(() => {
    const load = () => api.getHistory().then(setEntries);
    load();
    const sub = on("history-changed", load);
    return () => void sub.then((off) => off());
  }, []);

  const groups = useMemo(() => {
    const q = query.trim().toLowerCase();
    const list = q ? entries.filter((e) => e.text.toLowerCase().includes(q)) : entries;
    const out: [string, HistoryEntry[]][] = [];
    for (const e of list) {
      const day = relativeDay(e.at);
      if (out.length && out[out.length - 1][0] === day) out[out.length - 1][1].push(e);
      else out.push([day, [e]]);
    }
    return out;
  }, [entries, query]);

  const copy = async (e: HistoryEntry) => {
    await api.copyText(e.text);
    setCopied(e.id);
    setTimeout(() => setCopied(null), 1200);
  };

  const words = entries.reduce((n, e) => n + e.text.split(/\s+/).filter(Boolean).length, 0);

  return (
    <>
      <div className="history-bar">
        <input className="input search" placeholder="Search transcripts" value={query} onChange={(e) => setQuery(e.target.value)} />
        {entries.length > 0 && (
          <Button variant="ghost" size="sm" onClick={() => confirm("Delete all transcripts?") && api.clearHistory()}>
            Clear all
          </Button>
        )}
      </div>
      {entries.length > 0 && (
        <p className="muted small">
          {entries.length} dictations · {words.toLocaleString()} words
        </p>
      )}

      {groups.length === 0 && (
        <div className="empty">
          <div className="empty-icon">🎙️</div>
          <div>{query ? "Nothing matches your search." : "Your dictations will show up here."}</div>
        </div>
      )}

      {groups.map(([day, list]) => (
        <section className="section" key={day}>
          <h3 className="section-title">{day}</h3>
          <div className="card">
            {list.map((e) => (
              <div className="history-item" key={e.id}>
                <div className="history-time">{time(e.at)}</div>
                <div className="history-text">
                  {e.text}
                  {e.raw.trim() !== e.text.trim() && <div className="history-raw">Heard: {e.raw}</div>}
                </div>
                <div className="history-actions">
                  <Button size="sm" variant="ghost" onClick={() => copy(e)}>
                    {copied === e.id ? "Copied" : "Copy"}
                  </Button>
                  <Button size="sm" variant="ghost" onClick={() => api.deleteHistory(e.id)} aria-label="Delete">
                    ×
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </section>
      ))}
    </>
  );
}
