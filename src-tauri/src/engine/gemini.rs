use async_trait::async_trait;
use base64::Engine as Base64Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

use super::{ModelInfo, ProviderCategory, TranscriptionProvider, TranscriptionResult};
use crate::error::AppError;
use crate::models::segment::Segment;

pub struct GeminiProvider {
    api_key: String,
    client: Client,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    fn detect_mime_type(path: &str) -> Result<&'static str, AppError> {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "mp3" => Ok("audio/mp3"),
            "wav" => Ok("audio/wav"),
            "m4a" | "aac" => Ok("audio/mp4"),
            "ogg" | "oga" => Ok("audio/ogg"),
            "flac" => Ok("audio/flac"),
            "webm" => Ok("audio/webm"),
            "mp4" => Ok("video/mp4"),
            "mov" => Ok("video/quicktime"),
            _ => Err(AppError::Validation(format!(
                "Gemini がサポートしていない形式: .{ext}"
            ))),
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

            if let Some(parsed) = Self::try_parse_timestamped_line(trimmed) {
                segments.push(Segment {
                    id: format!("seg-{}", Uuid::new_v4()),
                    transcript_id: transcript_id.to_string(),
                    start_ms: parsed.0,
                    end_ms: parsed.1,
                    text: parsed.2,
                });
            } else {
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
        }

        segments
    }

    fn try_parse_timestamped_line(line: &str) -> Option<(i64, i64, String)> {
        if !line.starts_with('[') {
            return None;
        }
        let bracket_end = line.find(']')?;
        let timestamp_part = &line[1..bracket_end];
        let text = line[bracket_end + 1..].trim().to_string();

        let parts: Vec<&str> = timestamp_part.split("-->").collect();
        if parts.len() != 2 {
            return None;
        }

        let start = Self::parse_timestamp(parts[0].trim())?;
        let end = Self::parse_timestamp(parts[1].trim())?;
        Some((start, end, text))
    }

    fn parse_timestamp(ts: &str) -> Option<i64> {
        let parts: Vec<&str> = ts.split(':').collect();
        if parts.len() == 3 {
            let h: i64 = parts[0].parse().ok()?;
            let m: i64 = parts[1].parse().ok()?;
            let sec_parts: Vec<&str> = parts[2].split('.').collect();
            let s: i64 = sec_parts[0].parse().ok()?;
            let ms: i64 = if sec_parts.len() > 1 {
                let frac = sec_parts[1];
                let padded = format!("{:0<3}", &frac[..frac.len().min(3)]);
                padded.parse().ok()?
            } else {
                0
            };
            Some(h * 3600_000 + m * 60_000 + s * 1000 + ms)
        } else if parts.len() == 2 {
            let m: i64 = parts[0].parse().ok()?;
            let sec_parts: Vec<&str> = parts[1].split('.').collect();
            let s: i64 = sec_parts[0].parse().ok()?;
            let ms: i64 = if sec_parts.len() > 1 {
                let frac = sec_parts[1];
                let padded = format!("{:0<3}", &frac[..frac.len().min(3)]);
                padded.parse().ok()?
            } else {
                0
            };
            Some(m * 60_000 + s * 1000 + ms)
        } else {
            None
        }
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum GeminiPart {
    Text {
        text: String,
    },
    InlineData {
        inline_data: InlineData,
    },
}

#[derive(Serialize)]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiCandidateContent,
}

#[derive(Deserialize)]
struct GeminiCandidateContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: Option<String>,
}

#[async_trait]
impl TranscriptionProvider for GeminiProvider {
    fn provider_id(&self) -> &str {
        "google_gemini"
    }

    fn provider_name(&self) -> &str {
        "Google Gemini"
    }

    fn category(&self) -> ProviderCategory {
        ProviderCategory::ApiCloud
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gemini-3-flash-preview".to_string(),
                name: "Gemini 3 Flash (Preview)".to_string(),
                description: "最新 Flash - 高速・低コスト".to_string(),
                max_file_size_mb: 20,
            },
            ModelInfo {
                id: "gemini-3.1-pro-preview".to_string(),
                name: "Gemini 3.1 Pro (Preview)".to_string(),
                description: "最新 Pro - 最高精度".to_string(),
                max_file_size_mb: 20,
            },
            ModelInfo {
                id: "gemini-2.5-flash".to_string(),
                name: "Gemini 2.5 Flash".to_string(),
                description: "旧世代 Flash".to_string(),
                max_file_size_mb: 20,
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

        if audio_bytes.len() > 20 * 1024 * 1024 {
            return Err(AppError::Validation(
                "Gemini のインライン送信は 20MB が上限です。ファイルを小さくしてください。".to_string()
            ));
        }

        let audio_base64 = BASE64.encode(&audio_bytes);
        let mime_type = Self::detect_mime_type(audio_path)?;

        let lang_instruction = language
            .filter(|l| !l.is_empty() && *l != "auto")
            .map(|l| format!("The audio is in {l}. "))
            .unwrap_or_default();

        let prompt = format!(
            "{lang_instruction}Transcribe this audio file precisely. \
             Output each segment on its own line in this format:\n\
             [HH:MM:SS.mmm --> HH:MM:SS.mmm] transcribed text\n\n\
             If you cannot determine exact timestamps, estimate them based on speech pace. \
             Do not add any commentary, headers, or explanations. Only output the timestamped lines."
        );

        let request_body = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![
                    GeminiPart::InlineData {
                        inline_data: InlineData {
                            mime_type: mime_type.to_string(),
                            data: audio_base64,
                        },
                    },
                    GeminiPart::Text { text: prompt },
                ],
            }],
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model_id, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .json(&request_body)
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

        let api_response: GeminiResponse = response.json().await?;
        let raw_text = api_response
            .candidates
            .unwrap_or_default()
            .into_iter()
            .flat_map(|c| c.content.parts)
            .filter_map(|p| p.text)
            .collect::<Vec<_>>()
            .join("\n");

        let transcript_id = format!("tr-{}", Uuid::new_v4());
        let segments = Self::parse_segments(&raw_text, &transcript_id);

        let full_text = segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let duration_ms = segments.last().map(|s| s.end_ms).unwrap_or(0);

        Ok(TranscriptionResult {
            language: language.unwrap_or("auto").to_string(),
            duration_ms,
            segments,
            full_text,
        })
    }
}
