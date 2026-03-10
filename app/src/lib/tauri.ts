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
  lmStudioEndpoint?: string;
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

export interface YoutubeDownloadResult {
  path: string;
  fileName: string;
  videoId: string;
  title: string;
}

export async function downloadYoutube(
  url: string
): Promise<YoutubeDownloadResult> {
  return invoke<YoutubeDownloadResult>("download_youtube", { url });
}

export async function cleanupYoutubeTempFile(path: string): Promise<void> {
  return invoke<void>("cleanup_youtube_temp_file", { path });
}

export async function getSetting(key: string): Promise<string | null> {
  return invoke<string | null>("get_setting", { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke<void>("set_setting", { key, value });
}

export interface AddCustomModelRequest {
  providerId: string;
  modelId: string;
  name: string;
  description: string;
  maxFileSizeMb: number;
}

export interface CustomModelInfo {
  modelId: string;
  name: string;
  description: string;
  maxFileSizeMb: number;
}

export async function addCustomModel(
  request: AddCustomModelRequest
): Promise<void> {
  return invoke<void>("add_custom_model", { request });
}

export async function listCustomModels(
  providerId: string
): Promise<CustomModelInfo[]> {
  return invoke<CustomModelInfo[]>("list_custom_models", { providerId });
}

export async function deleteCustomModel(
  providerId: string,
  modelId: string
): Promise<void> {
  return invoke<void>("delete_custom_model", { providerId, modelId });
}
