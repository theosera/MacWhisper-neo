use crate::db::Database;
use crate::error::AppError;
use crate::models::transcript::{Transcript, TranscriptSummary};

#[tauri::command]
pub fn list_transcripts(
    db: tauri::State<'_, Database>,
) -> Result<Vec<TranscriptSummary>, AppError> {
    db.list_transcripts(100)
}

#[tauri::command]
pub fn get_transcript(
    id: String,
    db: tauri::State<'_, Database>,
) -> Result<Transcript, AppError> {
    db.get_transcript(&id)
}
