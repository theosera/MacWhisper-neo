import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { resolveDroppedFile } from "../lib/tauri";
import { useTranscription } from "../hooks/useTranscription";
import { useTranscriptStore } from "../stores/transcriptStore";

export function DropZone() {
  const [isDragOver, setIsDragOver] = useState(false);
  const { transcribe } = useTranscription();
  const isTranscribing = useTranscriptStore((s) => s.isTranscribing);

  useEffect(() => {
    const appWindow = getCurrentWindow();
    let cancelled = false;

    const setupListener = async () => {
      const unlisten = await appWindow.onDragDropEvent((event) => {
        if (cancelled) return;

        if (event.payload.type === "over") {
          setIsDragOver(true);
        } else if (event.payload.type === "leave") {
          setIsDragOver(false);
        } else if (event.payload.type === "drop") {
          setIsDragOver(false);
          const paths = event.payload.paths;
          if (paths.length > 0) {
            handleFileDrop(paths[0]);
          }
        }
      });

      return unlisten;
    };

    let unlistenFn: (() => void) | undefined;
    setupListener().then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlistenFn = fn;
      }
    });

    return () => {
      cancelled = true;
      unlistenFn?.();
    };
  }, []);

  async function handleFileDrop(filePath: string) {
    try {
      const info = await resolveDroppedFile(filePath);
      await transcribe(info.path);
    } catch (err) {
      useTranscriptStore.getState().setError(`${err}`);
    }
  }

  if (isTranscribing) {
    return (
      <div className="dropzone dropzone-processing">
        <div className="dropzone-spinner" />
        <p>Transcribing...</p>
      </div>
    );
  }

  return (
    <div className={`dropzone ${isDragOver ? "dropzone-active" : ""}`}>
      <p className="dropzone-icon">&#x1F3A4;</p>
      <p>Drop audio/video file here</p>
      <p className="dropzone-hint">mp3, wav, m4a, flac, ogg, mp4, mov</p>
    </div>
  );
}
