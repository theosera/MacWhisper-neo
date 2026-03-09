import { useTranscriptStore } from "../stores/transcriptStore";

function formatDuration(ms: number): string {
  const totalSec = Math.floor(ms / 1000);
  const h = Math.floor(totalSec / 3600);
  const m = Math.floor((totalSec % 3600) / 60);
  const s = totalSec % 60;
  if (h > 0) return `${h}h ${m}m ${s}s`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

export function MetaPanel() {
  const current = useTranscriptStore((s) => s.current);

  if (!current) {
    return (
      <div className="meta-panel">
        <h2>Info</h2>
        <p className="meta-empty">Select or create a transcription</p>
      </div>
    );
  }

  return (
    <div className="meta-panel">
      <h2>Info</h2>
      <dl className="meta-list">
        <dt>File</dt>
        <dd title={current.audioPath}>{current.fileName}</dd>

        <dt>Engine</dt>
        <dd>{current.engineId}</dd>

        <dt>Language</dt>
        <dd>{current.language}</dd>

        <dt>Created</dt>
        <dd>{new Date(current.createdAt).toLocaleString()}</dd>

        <dt>Duration</dt>
        <dd>{formatDuration(current.durationMs)}</dd>

        <dt>Processing</dt>
        <dd>{(current.processingTimeMs / 1000).toFixed(1)}s</dd>

        <dt>Segments</dt>
        <dd>{current.segments.length}</dd>

        <dt>Characters</dt>
        <dd>{current.fullText.length.toLocaleString()}</dd>
      </dl>
    </div>
  );
}
