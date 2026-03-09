import { useState, useCallback } from "react";
import { resolveDroppedFile } from "../lib/tauri";
import { useTranscription } from "../hooks/useTranscription";
import { useTranscriptStore } from "../stores/transcriptStore";

export function DropZone() {
  const [isDragOver, setIsDragOver] = useState(false);
  const { transcribe } = useTranscription();
  const isTranscribing = useTranscriptStore((s) => s.isTranscribing);

  const handleDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      setIsDragOver(false);

      const files = e.dataTransfer.files;
      if (files.length === 0) return;

      const filePath = (files[0] as unknown as { path?: string }).path;
      if (!filePath) {
        return;
      }

      try {
        const info = await resolveDroppedFile(filePath);
        await transcribe(info.path);
      } catch (err) {
        useTranscriptStore.getState().setError(`${err}`);
      }
    },
    [transcribe]
  );

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback(() => {
    setIsDragOver(false);
  }, []);

  if (isTranscribing) {
    return (
      <div className="dropzone dropzone-processing">
        <div className="dropzone-spinner" />
        <p>Transcribing...</p>
      </div>
    );
  }

  return (
    <div
      className={`dropzone ${isDragOver ? "dropzone-active" : ""}`}
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
    >
      <p className="dropzone-icon">&#x1F3A4;</p>
      <p>Drop audio/video file here</p>
      <p className="dropzone-hint">
        mp3, wav, m4a, flac, ogg, mp4, mov
      </p>
    </div>
  );
}
