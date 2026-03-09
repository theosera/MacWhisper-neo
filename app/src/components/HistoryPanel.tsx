import { useEffect } from "react";
import { useTranscriptStore } from "../stores/transcriptStore";
import { useTranscription } from "../hooks/useTranscription";

export function HistoryPanel() {
  const history = useTranscriptStore((s) => s.history);
  const selectedId = useTranscriptStore((s) => s.selectedId);
  const apiKey = useTranscriptStore((s) => s.apiKey);
  const setApiKey = useTranscriptStore((s) => s.setApiKey);
  const providers = useTranscriptStore((s) => s.providers);
  const selectedProviderId = useTranscriptStore((s) => s.selectedProviderId);
  const selectedModelId = useTranscriptStore((s) => s.selectedModelId);
  const setSelectedProvider = useTranscriptStore((s) => s.setSelectedProvider);
  const setSelectedModel = useTranscriptStore((s) => s.setSelectedModel);
  const { loadHistory, selectAndLoad, loadProviders } = useTranscription();

  useEffect(() => {
    loadHistory();
    loadProviders();
  }, [loadHistory, loadProviders]);

  const currentProvider = providers.find((p) => p.id === selectedProviderId);
  const models = currentProvider?.models ?? [];

  return (
    <div className="history-panel">
      <div className="history-header">
        <h2>History</h2>
      </div>

      <div className="settings-section">
        <label htmlFor="provider-select">Provider</label>
        <select
          id="provider-select"
          value={selectedProviderId}
          onChange={(e) => setSelectedProvider(e.target.value)}
        >
          {providers.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>

        <label htmlFor="model-select">Model</label>
        <select
          id="model-select"
          value={selectedModelId}
          onChange={(e) => setSelectedModel(e.target.value)}
        >
          {models.map((m) => (
            <option key={m.id} value={m.id}>
              {m.name}
            </option>
          ))}
        </select>

        <label htmlFor="api-key">API Key</label>
        <input
          id="api-key"
          type="password"
          placeholder="sk-..."
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
        />
      </div>

      <div className="history-list">
        {history.length === 0 ? (
          <p className="history-empty">No transcriptions yet</p>
        ) : (
          history.map((item) => (
            <button
              key={item.id}
              className={`history-item ${selectedId === item.id ? "history-item-selected" : ""}`}
              onClick={() => selectAndLoad(item.id)}
            >
              <span className="history-item-name">{item.fileName}</span>
              <span className="history-item-date">
                {new Date(item.createdAt).toLocaleDateString()}
              </span>
              <span className="history-item-preview">{item.preview}</span>
            </button>
          ))
        )}
      </div>
    </div>
  );
}
