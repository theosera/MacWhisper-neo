use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;

use crate::db::Database;
use crate::engine::{
    anthropic::AnthropicProvider, gemini::GeminiProvider, lm_studio::LmStudioProvider,
    openai_whisper::OpenAIWhisperProvider, ProviderRegistry, TranscriptionProvider,
};
use crate::error::AppError;
use crate::models::transcript::Transcript;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscribeRequest {
    pub audio_path: String,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub api_key: Option<String>,
    pub language: Option<String>,
    /// LM Studio 選択時のエンドポイント (デフォルト: http://localhost:1234)
    pub lm_studio_endpoint: Option<String>,
}

#[tauri::command]
pub async fn run_transcription(
    request: TranscribeRequest,
    db: tauri::State<'_, Database>,
    registry: tauri::State<'_, Mutex<ProviderRegistry>>,
) -> Result<Transcript, AppError> {
    let audio_path = request.audio_path.trim();
    if audio_path.is_empty() {
        return Err(AppError::Validation("audio_path is required".into()));
    }

    let path = std::path::Path::new(audio_path);
    if !path.is_file() {
        return Err(AppError::FileNotFound(audio_path.to_string()));
    }

    let provider_id = request.provider_id.as_deref().unwrap_or("anthropic");
    let model_id = request.model_id.as_deref().unwrap_or("claude-sonnet-4-6");

    // APIキー/エンドポイントが実行時に変わる provider はここで動的生成する
    let provider: Arc<dyn TranscriptionProvider> = match provider_id {
        "lm_studio" => {
            let endpoint = request
                .lm_studio_endpoint
                .unwrap_or_else(|| "http://localhost:1234".to_string());
            Arc::new(LmStudioProvider::new(endpoint))
        }
        "anthropic" => {
            let api_key = request
                .api_key
                .as_deref()
                .filter(|k| !k.trim().is_empty())
                .ok_or_else(|| {
                    AppError::Validation("Anthropic API key is required".to_string())
                })?;
            Arc::new(AnthropicProvider::new(api_key.to_string()))
        }
        "openai_whisper" => {
            let api_key = request
                .api_key
                .as_deref()
                .filter(|k| !k.trim().is_empty())
                .ok_or_else(|| AppError::Validation("OpenAI API key is required".to_string()))?;
            Arc::new(OpenAIWhisperProvider::new(api_key.to_string()))
        }
        "google_gemini" => {
            let api_key = request
                .api_key
                .as_deref()
                .filter(|k| !k.trim().is_empty())
                .ok_or_else(|| AppError::Validation("Google Gemini API key is required".to_string()))?;
            Arc::new(GeminiProvider::new(api_key.to_string()))
        }
        _ => {
            // whisper_cpp などキー不要 provider は既存 registry から取得
            let reg = registry.lock().unwrap();
            reg.get(provider_id)
                .ok_or_else(|| AppError::InvalidEngine(provider_id.to_string()))?
        }
    };

    let start = Instant::now();
    let result = provider
        .transcribe(audio_path, model_id, request.language.as_deref())
        .await?;
    let processing_time_ms = start.elapsed().as_millis() as i64;

    let transcript_id = format!("tr-{}", Uuid::new_v4());
    let created_at = chrono::Utc::now().to_rfc3339();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut segments = result.segments;
    for seg in &mut segments {
        seg.transcript_id = transcript_id.clone();
    }

    let engine_label = format!("{}/{}", provider_id, model_id);

    db.insert_transcript(
        &transcript_id,
        &created_at,
        audio_path,
        &file_name,
        &engine_label,
        &result.language,
        result.duration_ms,
        processing_time_ms,
        &result.full_text,
    )?;

    db.insert_segments(&segments)?;

    Ok(Transcript {
        id: transcript_id,
        created_at,
        audio_path: audio_path.to_string(),
        file_name,
        engine_id: engine_label,
        language: result.language,
        duration_ms: result.duration_ms,
        processing_time_ms,
        full_text: result.full_text,
        segments,
    })
}
