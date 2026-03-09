use serde::{Deserialize, Serialize};

use super::segment::Segment;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    pub id: String,
    pub created_at: String,
    pub audio_path: String,
    pub file_name: String,
    pub engine_id: String,
    pub language: String,
    pub duration_ms: i64,
    pub processing_time_ms: i64,
    pub full_text: String,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSummary {
    pub id: String,
    pub created_at: String,
    pub file_name: String,
    pub engine_id: String,
    pub language: String,
    pub preview: String,
}
