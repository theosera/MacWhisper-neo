import { useCallback } from "react";
import { useTranscriptStore } from "../stores/transcriptStore";
import {
  runTranscription,
  listTranscripts,
  getTranscript,
  listProviders,
  getSetting,
  setSetting,
  downloadYoutube,
  cleanupYoutubeTempFile,
} from "../lib/tauri";

const SETTING_KEYS = {
  API_KEYS: "api_keys_json",
  PROVIDER_ID: "selected_provider_id",
  MODEL_ID: "selected_model_id",
  LM_STUDIO_ENDPOINT: "lm_studio_endpoint",
  LANGUAGE: "language",
} as const;

/** API キー不要なプロバイダ */
const NO_KEY_PROVIDERS = new Set(["lm_studio", "whisper_cpp"]);

function needsApiKey(providerId: string): boolean {
  return !NO_KEY_PROVIDERS.has(providerId);
}

export function useTranscription() {
  const setHistory = useTranscriptStore((s) => s.setHistory);
  const setError = useTranscriptStore((s) => s.setError);
  const setProviders = useTranscriptStore((s) => s.setProviders);
  const setCurrent = useTranscriptStore((s) => s.setCurrent);
  const selectTranscript = useTranscriptStore((s) => s.selectTranscript);
  const setTranscribing = useTranscriptStore((s) => s.setTranscribing);
  const addToHistory = useTranscriptStore((s) => s.addToHistory);
  const setApiKeys = useTranscriptStore((s) => s.setApiKeys);
  const setLanguage = useTranscriptStore((s) => s.setLanguage);
  const setSelectedProvider = useTranscriptStore((s) => s.setSelectedProvider);
  const setSelectedModel = useTranscriptStore((s) => s.setSelectedModel);
  const setLmStudioEndpoint = useTranscriptStore((s) => s.setLmStudioEndpoint);

  const loadHistory = useCallback(async () => {
    try {
      const history = await listTranscripts();
      setHistory(history);
    } catch (e) {
      setError(`Failed to load history: ${e}`);
    }
  }, [setHistory, setError]);

  const loadProviders = useCallback(async () => {
    try {
      const providers = await listProviders();
      setProviders(providers);
    } catch (e) {
      setError(`Failed to load providers: ${e}`);
    }
  }, [setProviders, setError]);

  const loadSettings = useCallback(async () => {
    try {
      const [apiKeysJson, providerId, modelId, endpoint, language] =
        await Promise.all([
          getSetting(SETTING_KEYS.API_KEYS),
          getSetting(SETTING_KEYS.PROVIDER_ID),
          getSetting(SETTING_KEYS.MODEL_ID),
          getSetting(SETTING_KEYS.LM_STUDIO_ENDPOINT),
          getSetting(SETTING_KEYS.LANGUAGE),
        ]);

      if (apiKeysJson) {
        try {
          const parsed = JSON.parse(apiKeysJson);
          if (typeof parsed === "object" && parsed !== null) {
            setApiKeys(parsed);
          }
        } catch {
          // JSON parse 失敗時は旧形式（単一キー）を anthropic にマッピング
          setApiKeys({ anthropic: apiKeysJson });
        }
      }
      if (providerId) setSelectedProvider(providerId);
      if (modelId) setSelectedModel(modelId);
      if (endpoint) setLmStudioEndpoint(endpoint);
      if (language) setLanguage(language);
    } catch {
      // 初回起動時など
    }
  }, [setApiKeys, setSelectedProvider, setSelectedModel, setLmStudioEndpoint, setLanguage]);

  const saveSettings = useCallback(async () => {
    const { apiKeys, selectedProviderId, selectedModelId, lmStudioEndpoint, language } =
      useTranscriptStore.getState();
    try {
      await Promise.all([
        setSetting(SETTING_KEYS.API_KEYS, JSON.stringify(apiKeys)),
        setSetting(SETTING_KEYS.PROVIDER_ID, selectedProviderId),
        setSetting(SETTING_KEYS.MODEL_ID, selectedModelId),
        setSetting(SETTING_KEYS.LM_STUDIO_ENDPOINT, lmStudioEndpoint),
        setSetting(SETTING_KEYS.LANGUAGE, language),
      ]);
    } catch {
      // サイレント
    }
  }, []);

  const selectAndLoad = useCallback(
    async (id: string) => {
      selectTranscript(id);
      try {
        const transcript = await getTranscript(id);
        setCurrent(transcript);
        setError(null);
      } catch (e) {
        setError(`Failed to load transcript: ${e}`);
      }
    },
    [selectTranscript, setCurrent, setError]
  );

  const transcribe = useCallback(
    async (audioPath: string) => {
      const { apiKeys, selectedProviderId, selectedModelId, lmStudioEndpoint, language } =
        useTranscriptStore.getState();

      const isLocal = NO_KEY_PROVIDERS.has(selectedProviderId);
      const apiKey = apiKeys[selectedProviderId] ?? "";

      if (!isLocal && !apiKey) {
        setError(`${selectedProviderId} の API キーが未設定です。左パネルで入力してください。`);
        return;
      }

      setTranscribing(true);
      setError(null);
      try {
        const transcript = await runTranscription({
          audioPath,
          apiKey: isLocal ? undefined : apiKey,
          providerId: selectedProviderId,
          modelId: selectedModelId,
          lmStudioEndpoint: selectedProviderId === "lm_studio" ? lmStudioEndpoint : undefined,
          language: language !== "auto" ? language : undefined,
        });
        setCurrent(transcript);
        selectTranscript(transcript.id);
        addToHistory({
          id: transcript.id,
          createdAt: transcript.createdAt,
          fileName: transcript.fileName,
          engineId: transcript.engineId,
          language: transcript.language,
          preview: transcript.fullText.slice(0, 100),
        });
      } catch (e) {
        setError(`Transcription failed: ${e}`);
      } finally {
        setTranscribing(false);
      }
    },
    [setError, setTranscribing, setCurrent, selectTranscript, addToHistory]
  );

  const transcribeYoutube = useCallback(
    async (url: string) => {
      setTranscribing(true);
      setError(null);
      let downloadedPath: string | null = null;
      try {
        const result = await downloadYoutube(url);
        downloadedPath = result.path;
        const { apiKeys, selectedProviderId, selectedModelId, lmStudioEndpoint, language } =
          useTranscriptStore.getState();

        const isLocal = NO_KEY_PROVIDERS.has(selectedProviderId);
        const apiKey = apiKeys[selectedProviderId] ?? "";

        if (!isLocal && !apiKey) {
          setError(`${selectedProviderId} の API キーが未設定です。左パネルで入力してください。`);
          return;
        }

        const transcript = await runTranscription({
          audioPath: result.path,
          apiKey: isLocal ? undefined : apiKey,
          providerId: selectedProviderId,
          modelId: selectedModelId,
          lmStudioEndpoint: selectedProviderId === "lm_studio" ? lmStudioEndpoint : undefined,
          language: language !== "auto" ? language : undefined,
        });
        setCurrent(transcript);
        selectTranscript(transcript.id);
        addToHistory({
          id: transcript.id,
          createdAt: transcript.createdAt,
          fileName: result.title || transcript.fileName,
          engineId: transcript.engineId,
          language: transcript.language,
          preview: transcript.fullText.slice(0, 100),
        });
      } catch (e) {
        setError(`YouTube transcription failed: ${e}`);
      } finally {
        if (downloadedPath) {
          try {
            await cleanupYoutubeTempFile(downloadedPath);
          } catch {
            // 一時ファイル削除失敗は致命的ではない
          }
        }
        setTranscribing(false);
      }
    },
    [setError, setTranscribing, setCurrent, selectTranscript, addToHistory]
  );

  return {
    loadHistory,
    loadProviders,
    loadSettings,
    saveSettings,
    selectAndLoad,
    transcribe,
    transcribeYoutube,
    needsApiKey,
  };
}
