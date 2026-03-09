import { useCallback } from "react";
import { useTranscriptStore } from "../stores/transcriptStore";
import {
  runTranscription,
  listTranscripts,
  getTranscript,
} from "../lib/tauri";

export function useTranscription() {
  const store = useTranscriptStore();

  const loadHistory = useCallback(async () => {
    try {
      const history = await listTranscripts();
      store.setHistory(history);
    } catch (e) {
      store.setError(`Failed to load history: ${e}`);
    }
  }, [store]);

  const selectAndLoad = useCallback(
    async (id: string) => {
      store.selectTranscript(id);
      try {
        const transcript = await getTranscript(id);
        store.setCurrent(transcript);
        store.setError(null);
      } catch (e) {
        store.setError(`Failed to load transcript: ${e}`);
      }
    },
    [store]
  );

  const transcribe = useCallback(
    async (audioPath: string) => {
      if (!store.apiKey) {
        store.setError(
          "API key is required. Enter your Anthropic API key in the settings panel."
        );
        return;
      }
      store.setTranscribing(true);
      store.setError(null);
      try {
        const transcript = await runTranscription({
          audioPath,
          apiKey: store.apiKey,
          engineId: "anthropic",
        });
        store.setCurrent(transcript);
        store.selectTranscript(transcript.id);
        store.addToHistory({
          id: transcript.id,
          createdAt: transcript.createdAt,
          fileName: transcript.fileName,
          engineId: transcript.engineId,
          language: transcript.language,
          preview: transcript.fullText.slice(0, 100),
        });
      } catch (e) {
        store.setError(`Transcription failed: ${e}`);
      } finally {
        store.setTranscribing(false);
      }
    },
    [store]
  );

  return { loadHistory, selectAndLoad, transcribe };
}
