import { useEffect, useState } from "react";
import { useTranscriptStore } from "../stores/transcriptStore";
import { useTranscription } from "../hooks/useTranscription";
import { AddModelModal } from "./AddModelModal";

const LANGUAGE_OPTIONS = [
  { value: "auto", label: "Auto Detect" },
  { value: "ja", label: "日本語" },
  { value: "en", label: "English" },
  { value: "zh", label: "中文" },
  { value: "ko", label: "한국어" },
  { value: "fr", label: "Français" },
  { value: "de", label: "Deutsch" },
  { value: "es", label: "Español" },
  { value: "pt", label: "Português" },
];

const API_KEY_PLACEHOLDERS: Record<string, string> = {
  anthropic: "sk-ant-...",
  openai_whisper: "sk-...",
  google_gemini: "AIza...",
};

export function HistoryPanel() {
  const history = useTranscriptStore((s) => s.history);
  const selectedId = useTranscriptStore((s) => s.selectedId);
  const apiKeys = useTranscriptStore((s) => s.apiKeys);
  const setApiKey = useTranscriptStore((s) => s.setApiKey);
  const language = useTranscriptStore((s) => s.language);
  const setLanguage = useTranscriptStore((s) => s.setLanguage);
  const providers = useTranscriptStore((s) => s.providers);
  const selectedProviderId = useTranscriptStore((s) => s.selectedProviderId);
  const selectedModelId = useTranscriptStore((s) => s.selectedModelId);
  const lmStudioEndpoint = useTranscriptStore((s) => s.lmStudioEndpoint);
  const setSelectedProvider = useTranscriptStore((s) => s.setSelectedProvider);
  const setSelectedModel = useTranscriptStore((s) => s.setSelectedModel);
  const setLmStudioEndpoint = useTranscriptStore((s) => s.setLmStudioEndpoint);
  const { loadHistory, selectAndLoad, loadProviders, loadSettings, saveSettings, needsApiKey } =
    useTranscription();

  const [showAddModelModal, setShowAddModelModal] = useState(false);

  const isLmStudio = selectedProviderId === "lm_studio";
  const showApiKeyInput = needsApiKey(selectedProviderId);
  const currentApiKey = apiKeys[selectedProviderId] ?? "";

  useEffect(() => {
    loadHistory();
    loadProviders();
    loadSettings();
  }, [loadHistory, loadProviders, loadSettings]);

  const currentProvider = providers.find((p) => p.id === selectedProviderId);
  const models = currentProvider?.models ?? [];

  const handleApiKeyChange = (value: string) => {
    setApiKey(selectedProviderId, value);
  };
  const handleApiKeyBlur = () => saveSettings();
  const handleProviderChange = (value: string) => {
    setSelectedProvider(value);
    setTimeout(saveSettings, 0);
  };
  const handleModelChange = (value: string) => {
    setSelectedModel(value);
    setTimeout(saveSettings, 0);
  };
  const handleLanguageChange = (value: string) => {
    setLanguage(value);
    setTimeout(saveSettings, 0);
  };
  const handleEndpointBlur = () => saveSettings();

  return (
    <div className="history-panel">
      <div className="history-header">
        <h2>MacWhisper Neo</h2>
      </div>

      <div className="settings-section">
        <label htmlFor="provider-select">Provider</label>
        <select
          id="provider-select"
          value={selectedProviderId}
          onChange={(e) => handleProviderChange(e.target.value)}
        >
          {providers.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>

        <div className="model-select-row">
          <label htmlFor="model-select">Model</label>
          {showApiKeyInput && (
            <button
              type="button"
              className="add-model-btn"
              onClick={() => setShowAddModelModal(true)}
              title="カスタムモデルを追加"
            >
              +
            </button>
          )}
        </div>
        <select
          id="model-select"
          value={selectedModelId}
          onChange={(e) => handleModelChange(e.target.value)}
        >
          {models.map((m) => (
            <option key={m.id} value={m.id}>
              {m.name}
            </option>
          ))}
        </select>

        <label htmlFor="language-select">Language</label>
        <select
          id="language-select"
          value={language}
          onChange={(e) => handleLanguageChange(e.target.value)}
        >
          {LANGUAGE_OPTIONS.map((l) => (
            <option key={l.value} value={l.value}>
              {l.label}
            </option>
          ))}
        </select>

        {isLmStudio ? (
          <>
            <label htmlFor="lm-studio-endpoint">LM Studio Endpoint</label>
            <input
              id="lm-studio-endpoint"
              type="text"
              placeholder="http://localhost:1234"
              value={lmStudioEndpoint}
              onChange={(e) => setLmStudioEndpoint(e.target.value)}
              onBlur={handleEndpointBlur}
            />
            <p className="settings-hint">
              LM Studio を起動し、音声認識モデルをロードしてください
            </p>
          </>
        ) : showApiKeyInput ? (
          <>
            <label htmlFor="api-key">
              {currentProvider?.name ?? selectedProviderId} API Key
            </label>
            <input
              id="api-key"
              type="password"
              placeholder={API_KEY_PLACEHOLDERS[selectedProviderId] ?? "API Key..."}
              value={currentApiKey}
              onChange={(e) => handleApiKeyChange(e.target.value)}
              onBlur={handleApiKeyBlur}
            />
          </>
        ) : null}
      </div>

      <div className="history-header history-list-header">
        <h3>履歴</h3>
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
                {new Date(item.createdAt).toLocaleDateString("ja-JP")}
              </span>
              <span className="history-item-preview">{item.preview}</span>
            </button>
          ))
        )}
      </div>

      {showAddModelModal && currentProvider && (
        <AddModelModal
          providerId={currentProvider.id}
          providerName={currentProvider.name}
          onClose={() => setShowAddModelModal(false)}
          onSuccess={() => {
            setShowAddModelModal(false);
            loadProviders();
          }}
        />
      )}
    </div>
  );
}
