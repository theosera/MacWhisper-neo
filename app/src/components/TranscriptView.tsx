import { useTranscriptStore } from "../stores/transcriptStore";
import { DropZone } from "./DropZone";
import { ExportMenu } from "./ExportMenu";

function formatMs(ms: number): string {
  const totalSec = Math.floor(ms / 1000);
  const h = Math.floor(totalSec / 3600);
  const m = Math.floor((totalSec % 3600) / 60);
  const s = totalSec % 60;
  if (h > 0) {
    return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  }
  return `${m}:${String(s).padStart(2, "0")}`;
}

export function TranscriptView() {
  const current = useTranscriptStore((s) => s.current);
  const error = useTranscriptStore((s) => s.error);
  const isTranscribing = useTranscriptStore((s) => s.isTranscribing);

  return (
    <div className="transcript-view">
      <div className="transcript-header">
        <h2>{current ? current.fileName : "MacWhisper Neo"}</h2>
        {current && <ExportMenu />}
      </div>

      {error && (
        <div className="error-banner">
          <span>{error}</span>
          <button onClick={() => useTranscriptStore.getState().setError(null)}>
            Dismiss
          </button>
        </div>
      )}

      {!current && !isTranscribing && <DropZone />}

      {isTranscribing && (
        <div className="transcript-loading">
          <div className="dropzone-spinner" />
          <p>Transcribing audio...</p>
        </div>
      )}

      {current && (
        <div className="segments-list">
          {current.segments.map((seg) => (
            <div key={seg.id} className="segment-row">
              <div className="segment-time">
                <span className="time-start">{formatMs(seg.startMs)}</span>
                <span className="time-sep">-</span>
                <span className="time-end">{formatMs(seg.endMs)}</span>
              </div>
              <div className="segment-text">{seg.text}</div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
