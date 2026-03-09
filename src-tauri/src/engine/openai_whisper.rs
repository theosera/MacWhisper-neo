use async_trait::async_trait;
use reqwest::Client;
use std::path::Path;
use uuid::Uuid;

use super::{TranscriptionProvider, TranscriptionResult, ModelInfo, ProviderCategory};
use crate::error::AppError;
use crate::models::segment::Segment;

pub struct OpenAIWhisperProvider {
    api_key: String,
    client: Client,
}

impl OpenAIWhisperProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
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
impl TranscriptionProvider for OpenAIWhisperProvider {
    fn provider_id(&self) -> &str {
        "openai_whisper"
    }

    fn provider_name(&self) -> &str {
        "OpenAI Whisper"
    }

    fn category(&self) -> ProviderCategory {
        ProviderCategory::ApiCloud
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: "whisper-1".to_string(),
            name: "Whisper v3 (Latest)".to_string(),
            description: "Latest Whisper model - best accuracy".to_string(),
            max_file_size_mb: 25,
        }]
    }

    async fn transcribe(
        &self,
        audio_path: &str,
        _model_id: &str,
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

        let form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(audio_bytes)
                    .file_name(file_name.to_string()),
            )
            .text("model", "whisper-1");

        let form = if let Some(lang) = language.filter(|l| !l.is_empty() && *l != "auto") {
            form.text("language", lang.to_string())
        } else {
            form
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        #[derive(serde::Deserialize)]
        struct WhisperResponse {
            text: String,
        }

        let api_response: WhisperResponse = response.json().await?;
        let raw_text = api_response.text;

        let transcript_id = format!("tr-{}", Uuid::new_v4());
        let segments = Self::parse_segments(&raw_text, &transcript_id);

        let duration_ms = segments.last().map(|s| s.end_ms).unwrap_or(0);

        Ok(TranscriptionResult {
            language: language.unwrap_or("auto").to_string(),
            duration_ms,
            segments,
            full_text: raw_text,
        })
    }
}
