import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { resolveDroppedFile } from "../lib/tauri";
import { useTranscription } from "../hooks/useTranscription";
import { useTranscriptStore } from "../stores/transcriptStore";

export function DropZone() {
  const [isDragOver, setIsDragOver] = useState(false);
  const [youtubeUrl, setYoutubeUrl] = useState("");
  const [isDownloading, setIsDownloading] = useState(false);
  const { transcribe, transcribeYoutube } = useTranscription();
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

  async function handleYoutubeSubmit() {
    const url = youtubeUrl.trim();
    if (!url) return;

    setIsDownloading(true);
    try {
      await transcribeYoutube(url);
      setYoutubeUrl("");
    } finally {
      setIsDownloading(false);
    }
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter" && !isDownloading && !isTranscribing) {
      handleYoutubeSubmit();
    }
  }

  if (isTranscribing || isDownloading) {
    return (
      <div className="dropzone dropzone-processing">
        <div className="dropzone-spinner" />
        <p>{isDownloading ? "YouTube から音声をダウンロード中..." : "文字起こし中..."}</p>
      </div>
    );
  }

  return (
    <div className="dropzone-container">
      <div className={`dropzone ${isDragOver ? "dropzone-active" : ""}`}>
        <p className="dropzone-icon">&#x1F3A4;</p>
        <p>音声/動画ファイルをドロップ</p>
        <p className="dropzone-hint">
          Audio: mp3, wav, m4a, aac, ogg, flac
          <br />
          Video: mp4, mov, mkv, webm, avi
        </p>
      </div>

      <div className="youtube-section">
        <div className="youtube-divider">
          <span>または</span>
        </div>
        <div className="youtube-input-row">
          <input
            type="text"
            className="youtube-url-input"
            placeholder="YouTube URL を貼り付け..."
            value={youtubeUrl}
            onChange={(e) => setYoutubeUrl(e.target.value)}
            onKeyDown={handleKeyDown}
          />
          <button
            className="youtube-submit-btn"
            onClick={handleYoutubeSubmit}
            disabled={!youtubeUrl.trim()}
          >
            文字起こし
          </button>
        </div>
        <p className="dropzone-hint">
          watch?v=, youtu.be/, shorts/, embed/ に対応
        </p>
      </div>
    </div>
  );
}
