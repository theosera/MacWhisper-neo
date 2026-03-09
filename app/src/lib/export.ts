import type { ExportFormat, Transcript } from "../types";

function formatMs(ms: number): string {
  const totalSec = Math.floor(ms / 1000);
  const h = Math.floor(totalSec / 3600);
  const m = Math.floor((totalSec % 3600) / 60);
  const s = totalSec % 60;
  const millis = ms % 1000;
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}.${String(millis).padStart(3, "0")}`;
}

export function exportTranscript(
  transcript: Transcript,
  format: ExportFormat
): string {
  switch (format) {
    case "markdown":
      return toMarkdown(transcript);
    case "txt":
      return toTxt(transcript);
    case "json":
      return toJson(transcript);
  }
}

function toMarkdown(t: Transcript): string {
  const lines: string[] = [
    `# ${t.fileName}`,
    "",
    `- **Engine**: ${t.engineId}`,
    `- **Language**: ${t.language}`,
    `- **Date**: ${new Date(t.createdAt).toLocaleString()}`,
    `- **Duration**: ${formatMs(t.durationMs)}`,
    `- **Processing Time**: ${(t.processingTimeMs / 1000).toFixed(1)}s`,
    "",
    "---",
    "",
  ];

  for (const seg of t.segments) {
    lines.push(
      `**[${formatMs(seg.startMs)} - ${formatMs(seg.endMs)}]**  `
    );
    lines.push(seg.text);
    lines.push("");
  }

  return lines.join("\n");
}

function toTxt(t: Transcript): string {
  const lines: string[] = [];
  for (const seg of t.segments) {
    lines.push(
      `[${formatMs(seg.startMs)} --> ${formatMs(seg.endMs)}] ${seg.text}`
    );
  }
  return lines.join("\n");
}

function toJson(t: Transcript): string {
  return JSON.stringify(t, null, 2);
}

export function getFileExtension(format: ExportFormat): string {
  switch (format) {
    case "markdown":
      return "md";
    case "txt":
      return "txt";
    case "json":
      return "json";
  }
}
