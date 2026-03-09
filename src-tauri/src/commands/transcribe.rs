use serde::Deserialize;
use std::time::Instant;
use uuid::Uuid;

use crate::db::Database;
use crate::engine;
use crate::error::AppError;
use crate::models::transcript::Transcript;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscribeRequest {
    pub audio_path: String,
    pub engine_id: Option<String>,
    pub api_key: String,
    pub language: Option<String>,
}

#[tauri::command]
pub async fn run_transcription(
    request: TranscribeRequest,
    db: tauri::State<'_, Database>,
) -> Result<Transcript, AppError> {
    let audio_path = request.audio_path.trim();
    if audio_path.is_empty() {
        return Err(AppError::Validation("audio_path is required".into()));
    }

    let path = std::path::Path::new(audio_path);
    if !path.is_file() {
        return Err(AppError::FileNotFound(audio_path.to_string()));
    }

    let engine_id = request.engine_id.as_deref().unwrap_or("anthropic");
    let engine = engine::create_engine(engine_id, &request.api_key)?;

    let start = Instant::now();
    let result = engine
        .transcribe(audio_path, request.language.as_deref())
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

    db.insert_transcript(
        &transcript_id,
        &created_at,
        audio_path,
        &file_name,
        engine_id,
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
        engine_id: engine_id.to_string(),
        language: result.language,
        duration_ms: result.duration_ms,
        processing_time_ms,
        full_text: result.full_text,
        segments,
    })
}
