use async_trait::async_trait;
use base64::Engine as Base64Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

use super::{TranscriptionEngine, TranscriptionResult};
use crate::error::AppError;
use crate::models::segment::Segment;

pub struct AnthropicEngine {
    api_key: String,
    client: Client,
}

impl AnthropicEngine {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    fn detect_media_type(path: &str) -> Result<&'static str, AppError> {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "mp3" => Ok("audio/mpeg"),
            "wav" => Ok("audio/wav"),
            "m4a" | "aac" => Ok("audio/mp4"),
            "ogg" | "oga" => Ok("audio/ogg"),
            "flac" => Ok("audio/flac"),
            "webm" => Ok("audio/webm"),
            "mp4" => Ok("video/mp4"),
            "mov" => Ok("video/quicktime"),
            _ => Err(AppError::Validation(format!(
                "Unsupported audio format: .{ext}"
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

    /// Try to parse "[HH:MM:SS.mmm --> HH:MM:SS.mmm] text" format
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
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: Vec<ContentBlock>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "document")]
    Document {
        source: DocumentSource,
    },
}

#[derive(Serialize)]
struct DocumentSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ResponseContent>,
}

#[derive(Deserialize)]
struct ResponseContent {
    text: Option<String>,
}

#[async_trait]
impl TranscriptionEngine for AnthropicEngine {
    fn engine_id(&self) -> &str {
        "anthropic"
    }

    async fn transcribe(
        &self,
        audio_path: &str,
        language: Option<&str>,
    ) -> Result<TranscriptionResult, AppError> {
        if !Path::new(audio_path).is_file() {
            return Err(AppError::FileNotFound(audio_path.to_string()));
        }

        let audio_bytes = tokio::fs::read(audio_path).await?;
        let audio_base64 = BASE64.encode(&audio_bytes);
        let media_type = Self::detect_media_type(audio_path)?;

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

        let request_body = AnthropicRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 8192,
            messages: vec![Message {
                role: "user".to_string(),
                content: vec![
                    ContentBlock::Document {
                        source: DocumentSource {
                            source_type: "base64".to_string(),
                            media_type: media_type.to_string(),
                            data: audio_base64,
                        },
                    },
                    ContentBlock::Text { text: prompt },
                ],
            }],
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
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

        let api_response: AnthropicResponse = response.json().await?;
        let raw_text = api_response
            .content
            .into_iter()
            .filter_map(|c| c.text)
            .collect::<Vec<_>>()
            .join("\n");

        let transcript_id = format!("tr-{}", Uuid::new_v4());
        let segments = Self::parse_segments(&raw_text, &transcript_id);

        let full_text = segments.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join("\n");

        let duration_ms = segments.last().map(|s| s.end_ms).unwrap_or(0);

        Ok(TranscriptionResult {
            language: language.unwrap_or("auto").to_string(),
            duration_ms,
            segments,
            full_text,
        })
    }
}
