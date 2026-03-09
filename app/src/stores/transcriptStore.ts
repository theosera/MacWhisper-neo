import { create } from "zustand";
import type { Transcript, TranscriptSummary, ProviderInfo } from "../types";

interface TranscriptState {
  history: TranscriptSummary[];
  selectedId: string | null;
  current: Transcript | null;
  isTranscribing: boolean;
  error: string | null;
  apiKey: string;

  providers: ProviderInfo[];
  selectedProviderId: string;
  selectedModelId: string;

  setHistory: (history: TranscriptSummary[]) => void;
  selectTranscript: (id: string | null) => void;
  setCurrent: (transcript: Transcript | null) => void;
  setTranscribing: (value: boolean) => void;
  setError: (error: string | null) => void;
  setApiKey: (key: string) => void;
  addToHistory: (summary: TranscriptSummary) => void;
  setProviders: (providers: ProviderInfo[]) => void;
  setSelectedProvider: (providerId: string) => void;
  setSelectedModel: (modelId: string) => void;
}

export const useTranscriptStore = create<TranscriptState>((set) => ({
  history: [],
  selectedId: null,
  current: null,
  isTranscribing: false,
  error: null,
  apiKey: "",

  providers: [],
  selectedProviderId: "anthropic",
  selectedModelId: "claude-sonnet-4-20250514",

  setHistory: (history) => set({ history }),
  selectTranscript: (id) => set({ selectedId: id }),
  setCurrent: (transcript) => set({ current: transcript }),
  setTranscribing: (value) => set({ isTranscribing: value }),
  setError: (error) => set({ error }),
  setApiKey: (key) => set({ apiKey: key }),
  addToHistory: (summary) =>
    set((state) => ({ history: [summary, ...state.history] })),
  setProviders: (providers) => set({ providers }),
  setSelectedProvider: (providerId) => set((state) => {
    const provider = state.providers.find((p) => p.id === providerId);
    const firstModel = provider?.models[0]?.id ?? "";
    return { selectedProviderId: providerId, selectedModelId: firstModel };
  }),
  setSelectedModel: (modelId) => set({ selectedModelId: modelId }),
}));
