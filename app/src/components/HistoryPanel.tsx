import { useEffect } from "react";
import { useTranscriptStore } from "../stores/transcriptStore";
import { useTranscription } from "../hooks/useTranscription";

export function HistoryPanel() {
  const history = useTranscriptStore((s) => s.history);
  const selectedId = useTranscriptStore((s) => s.selectedId);
  const apiKey = useTranscriptStore((s) => s.apiKey);
  const setApiKey = useTranscriptStore((s) => s.setApiKey);
  const { loadHistory, selectAndLoad } = useTranscription();

  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  return (
    <div className="history-panel">
      <div className="history-header">
        <h2>History</h2>
      </div>

      <div className="api-key-section">
        <label htmlFor="api-key">API Key</label>
        <input
          id="api-key"
          type="password"
          placeholder="sk-ant-..."
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
