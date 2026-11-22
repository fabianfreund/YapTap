import { useState } from "react";
import { api, type Model } from "../lib/ipc";
import { bytes } from "../lib/format";
import { useStore } from "../settings/store";
import { Badge, Button, Dots, ProgressBar } from "./ui";

export function ModelCard({ model, compact }: { model: Model; compact?: boolean }) {
  const { settings, status, progress, update } = useStore();
  const [error, setError] = useState<string | null>(null);
  const selected = settings.model === model.id;
  const p = progress[model.id] ?? (model.downloading != null ? model.downloading / model.size : undefined);
  const downloading = p !== undefined;
  const loading = selected && status.engine.status === "loading";

  const download = async () => {
    setError(null);
    try {
      await api.downloadModel(model.id);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className={`model ${selected ? "is-selected" : ""}`}>
      <div className="model-head">
        <div>
          <div className="model-name">
            {model.name}
            {model.id === "parakeet-v3" && <Badge tone="accent">Recommended</Badge>}
            {selected && model.installed && (
              <Badge tone={loading ? "warn" : "ok"}>{loading ? "Loading…" : "In use"}</Badge>
            )}
          </div>
          <div className="model-desc">{model.description}</div>
        </div>
        <div className="model-actions">
          {model.installed ? (
            !selected ? (
              <>
                <Button size="sm" variant="primary" onClick={() => update({ model: model.id })}>
                  Use
                </Button>
                {!compact && (
                  <Button size="sm" variant="ghost" onClick={() => api.deleteModel(model.id)} title="Delete">
                    Delete
                  </Button>
                )}
              </>
            ) : null
          ) : downloading ? (
            <Button size="sm" variant="ghost" onClick={() => api.cancelDownload(model.id)}>
              Cancel
            </Button>
          ) : (
            <Button size="sm" variant={selected ? "primary" : "default"} onClick={download}>
              Download · {bytes(model.size)}
            </Button>
          )}
        </div>
      </div>
      {downloading && (
        <div className="model-progress">
          <ProgressBar value={p ?? 0} />
          <span>
            {Math.round((p ?? 0) * 100)}% of {bytes(model.size)}
          </span>
        </div>
      )}
      {!compact && !downloading && (
        <div className="model-meta">
          <span>Speed <Dots n={model.speed} /></span>
          <span>Accuracy <Dots n={model.accuracy} /></span>
          <span>{model.languages} languages</span>
          <span>{model.license}</span>
        </div>
      )}
      {selected && status.engine.status === "error" && <div className="field-error">{status.engine.error}</div>}
      {error && <div className="field-error">{error}</div>}
    </div>
  );
}
