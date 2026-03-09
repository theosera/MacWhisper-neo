export interface Segment {
  id: string;
  transcriptId: string;
  startMs: number;
  endMs: number;
  text: string;
}

export interface Transcript {
  id: string;
  createdAt: string;
  audioPath: string;
  fileName: string;
  engineId: string;
  language: string;
  durationMs: number;
  processingTimeMs: number;
  fullText: string;
  segments: Segment[];
}

export interface TranscriptSummary {
  id: string;
  createdAt: string;
  fileName: string;
  engineId: string;
  language: string;
  preview: string;
}

export interface FileInfo {
  path: string;
  fileName: string;
  extension: string;
  sizeBytes: number;
}

export type ExportFormat = "markdown" | "txt" | "json";

export interface ModelInfo {
  id: string;
  name: string;
  description: string;
  maxFileSizeMb: number;
}

export type ProviderCategory = "api_cloud" | "api_local" | "local_binary";

export interface ProviderInfo {
  id: string;
  name: string;
  category: ProviderCategory;
  models: ModelInfo[];
}
