use async_trait::async_trait;
use reqwest::Client;
use std::path::Path;
use uuid::Uuid;

use super::{ModelInfo, ProviderCategory, TranscriptionProvider, TranscriptionResult};
use crate::error::AppError;
use crate::models::segment::Segment;

pub struct LmStudioProvider {
    endpoint: String,
    client: Client,
}

impl LmStudioProvider {
    pub fn new(endpoint: String) -> Self {
        let endpoint = endpoint.trim_end_matches('/').to_string();
        Self {
            endpoint,
            client: Client::new(),
        }
    }

    fn parse_segments(text: &str, transcript_id: &str) -> Vec<Segment> {
        let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.is_empty() {
            return Vec::new();
        }

        let mut segments = Vec::new();
        let mut current_ms: i64 = 0;

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let words: Vec<&str> = trimmed.split_whitespace().collect();
            let estimated_duration = (words.len() as i64) * 300;
            segments.push(Segment {
                id: format!("seg-{}", Uuid::new_v4()),
                transcript_id: transcript_id.to_string(),
                start_ms: current_ms,
                end_ms: current_ms + estimated_duration,
                text: trimmed.to_string(),
            });
            current_ms += estimated_duration;
        }

        segments
    }
}

#[async_trait]
impl TranscriptionProvider for LmStudioProvider {
    fn provider_id(&self) -> &str {
        "lm_studio"
    }

    fn provider_name(&self) -> &str {
        "LM Studio (Local)"
    }

    fn category(&self) -> ProviderCategory {
        ProviderCategory::ApiLocal
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "auto".to_string(),
                name: "Auto (ロード済みモデル)".to_string(),
                description: "LM Studio で現在ロードされているモデルを使用".to_string(),
                max_file_size_mb: 200,
            },
            ModelInfo {
                id: "faster-whisper-large-v3".to_string(),
                name: "Faster Whisper Large v3".to_string(),
                description: "高精度 Whisper モデル (LM Studio でロード必要)".to_string(),
                max_file_size_mb: 200,
            },
            ModelInfo {
                id: "faster-whisper-medium".to_string(),
                name: "Faster Whisper Medium".to_string(),
                description: "バランス型 Whisper モデル (LM Studio でロード必要)".to_string(),
                max_file_size_mb: 200,
            },
            ModelInfo {
                id: "faster-whisper-small".to_string(),
                name: "Faster Whisper Small".to_string(),
                description: "高速 Whisper モデル (LM Studio でロード必要)".to_string(),
                max_file_size_mb: 200,
            },
        ]
    }

    async fn transcribe(
        &self,
        audio_path: &str,
        model_id: &str,
        language: Option<&str>,
    ) -> Result<TranscriptionResult, AppError> {
        if !Path::new(audio_path).is_file() {
            return Err(AppError::FileNotFound(audio_path.to_string()));
        }

        let audio_bytes = tokio::fs::read(audio_path).await?;
        let file_name = Path::new(audio_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio");

        // model_id が "auto" の場合は空文字列を渡す（LM Studio がロード済みモデルを使用）
        let model_param = if model_id == "auto" { "" } else { model_id };

        let mut form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(audio_bytes).file_name(file_name.to_string()),
        );

        if !model_param.is_empty() {
            form = form.text("model", model_param.to_string());
        }

        if let Some(lang) = language.filter(|l| !l.is_empty() && *l != "auto") {
            form = form.text("language", lang.to_string());
        }

        // verbose_json レスポンス形式でタイムスタンプ付きセグメントを取得
        form = form.text("response_format", "verbose_json");

        let url = format!("{}/v1/audio/transcriptions", self.endpoint);

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    AppError::Validation(format!(
                        "LM Studio に接続できません ({})。LM Studio が起動しているか確認してください。",
                        self.endpoint
                    ))
                } else {
                    AppError::Http(e)
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Api {
                status: status.as_u16(),
                message: format!(
                    "LM Studio エラー: {}。音声認識対応モデルがロードされているか確認してください。",
                    body
                ),
            });
        }

        // verbose_json と通常レスポンス両方に対応
        let raw_body = response.text().await?;

        #[derive(serde::Deserialize)]
        struct SimpleResponse {
            text: String,
        }

        #[derive(serde::Deserialize)]
        struct VerboseSegment {
            start: f64,
            end: f64,
            text: String,
        }

        #[derive(serde::Deserialize)]
        struct VerboseResponse {
            text: String,
            #[serde(default)]
            segments: Vec<VerboseSegment>,
            language: Option<String>,
        }

        let transcript_id = format!("tr-{}", Uuid::new_v4());

        // verbose_json として解析を試みる
        let (full_text, segments, detected_language) =
            if let Ok(verbose) = serde_json::from_str::<VerboseResponse>(&raw_body) {
                let segs: Vec<Segment> = if verbose.segments.is_empty() {
                    Self::parse_segments(&verbose.text, &transcript_id)
                } else {
                    verbose
                        .segments
                        .iter()
                        .map(|s| Segment {
                            id: format!("seg-{}", Uuid::new_v4()),
                            transcript_id: transcript_id.clone(),
                            start_ms: (s.start * 1000.0) as i64,
                            end_ms: (s.end * 1000.0) as i64,
                            text: s.text.trim().to_string(),
                        })
                        .collect()
                };
                let lang = verbose.language.unwrap_or_else(|| "auto".to_string());
                (verbose.text, segs, lang)
            } else if let Ok(simple) = serde_json::from_str::<SimpleResponse>(&raw_body) {
                // フォールバック: 通常の {"text": "..."} 形式
                let segs = Self::parse_segments(&simple.text, &transcript_id);
                (simple.text, segs, "auto".to_string())
            } else {
                return Err(AppError::Validation(
                    "LM Studio からの応答を解析できませんでした。音声認識対応モデルがロードされているか確認してください。".to_string()
                ));
            };

        let duration_ms = segments.last().map(|s| s.end_ms).unwrap_or(0);
        let lang = language
            .filter(|l| !l.is_empty() && *l != "auto")
            .map(|l| l.to_string())
            .unwrap_or(detected_language);

        Ok(TranscriptionResult {
            language: lang,
            duration_ms,
            segments,
            full_text,
        })
    }
}
