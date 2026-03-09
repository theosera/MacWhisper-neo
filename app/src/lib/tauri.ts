import { invoke } from "@tauri-apps/api/core";
import type {
  FileInfo,
  ProviderInfo,
  Transcript,
  TranscriptSummary,
} from "../types";

export interface TranscribeRequest {
  audioPath: string;
  providerId?: string;
  modelId?: string;
  apiKey?: string;
  language?: string;
}

export async function runTranscription(
  request: TranscribeRequest
): Promise<Transcript> {
  return invoke<Transcript>("run_transcription", { request });
}

export async function listTranscripts(): Promise<TranscriptSummary[]> {
  return invoke<TranscriptSummary[]>("list_transcripts");
}

export async function getTranscript(id: string): Promise<Transcript> {
  return invoke<Transcript>("get_transcript", { id });
}

export async function resolveDroppedFile(path: string): Promise<FileInfo> {
  return invoke<FileInfo>("resolve_dropped_file", { path });
}

export async function listProviders(): Promise<ProviderInfo[]> {
  return invoke<ProviderInfo[]>("list_providers");
}
