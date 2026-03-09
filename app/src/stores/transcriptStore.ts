import { create } from "zustand";
import type { Transcript, TranscriptSummary } from "../types";

interface TranscriptState {
  history: TranscriptSummary[];
  selectedId: string | null;
  current: Transcript | null;
  isTranscribing: boolean;
  error: string | null;
  apiKey: string;

  setHistory: (history: TranscriptSummary[]) => void;
  selectTranscript: (id: string | null) => void;
  setCurrent: (transcript: Transcript | null) => void;
  setTranscribing: (value: boolean) => void;
  setError: (error: string | null) => void;
  setApiKey: (key: string) => void;
  addToHistory: (summary: TranscriptSummary) => void;
}

export const useTranscriptStore = create<TranscriptState>((set) => ({
  history: [],
  selectedId: null,
  current: null,
  isTranscribing: false,
  error: null,
  apiKey: "",

  setHistory: (history) => set({ history }),
  selectTranscript: (id) => set({ selectedId: id }),
  setCurrent: (transcript) => set({ current: transcript }),
  setTranscribing: (value) => set({ isTranscribing: value }),
  setError: (error) => set({ error }),
  setApiKey: (key) => set({ apiKey: key }),
  addToHistory: (summary) =>
    set((state) => ({ history: [summary, ...state.history] })),
}));
