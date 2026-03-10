import { create } from "zustand";
import type { Transcript, TranscriptSummary, ProviderInfo } from "../types";

interface TranscriptState {
  history: TranscriptSummary[];
  selectedId: string | null;
  current: Transcript | null;
  isTranscribing: boolean;
  error: string | null;
  apiKeys: Record<string, string>;
  language: string;

  providers: ProviderInfo[];
  selectedProviderId: string;
  selectedModelId: string;
  lmStudioEndpoint: string;

  setHistory: (history: TranscriptSummary[]) => void;
  selectTranscript: (id: string | null) => void;
  setCurrent: (transcript: Transcript | null) => void;
  setTranscribing: (value: boolean) => void;
  setError: (error: string | null) => void;
  setApiKey: (providerId: string, key: string) => void;
  getApiKey: (providerId: string) => string;
  setLanguage: (lang: string) => void;
  addToHistory: (summary: TranscriptSummary) => void;
  setProviders: (providers: ProviderInfo[]) => void;
  setSelectedProvider: (providerId: string) => void;
  setSelectedModel: (modelId: string) => void;
  setLmStudioEndpoint: (endpoint: string) => void;
  setApiKeys: (keys: Record<string, string>) => void;
}

export const useTranscriptStore = create<TranscriptState>((set, get) => ({
  history: [],
  selectedId: null,
  current: null,
  isTranscribing: false,
  error: null,
  apiKeys: {},
  language: "auto",

  providers: [],
  selectedProviderId: "anthropic",
  selectedModelId: "claude-sonnet-4-6",
  lmStudioEndpoint: "http://localhost:1234",

  setHistory: (history) => set({ history }),
  selectTranscript: (id) => set({ selectedId: id }),
  setCurrent: (transcript) => set({ current: transcript }),
  setTranscribing: (value) => set({ isTranscribing: value }),
  setError: (error) => set({ error }),
  setApiKey: (providerId, key) =>
    set((state) => ({
      apiKeys: { ...state.apiKeys, [providerId]: key },
    })),
  getApiKey: (providerId) => get().apiKeys[providerId] ?? "",
  setApiKeys: (keys) => set({ apiKeys: keys }),
  setLanguage: (lang) => set({ language: lang }),
  addToHistory: (summary) =>
    set((state) => ({ history: [summary, ...state.history] })),
  setProviders: (providers) =>
    set((state) => {
      const update: Partial<TranscriptState> = { providers };
      const currentProviderValid = providers.some(
        (p) => p.id === state.selectedProviderId
      );
      if (!currentProviderValid && providers.length > 0) {
        const firstProvider = providers[0];
        update.selectedProviderId = firstProvider.id;
        update.selectedModelId = firstProvider.models[0]?.id ?? "";
      }
      return update;
    }),
  setSelectedProvider: (providerId) =>
    set((state) => {
      const provider = state.providers.find((p) => p.id === providerId);
      const firstModel = provider?.models[0]?.id ?? "";
      return { selectedProviderId: providerId, selectedModelId: firstModel };
    }),
  setSelectedModel: (modelId) => set({ selectedModelId: modelId }),
  setLmStudioEndpoint: (endpoint) => set({ lmStudioEndpoint: endpoint }),
}));
