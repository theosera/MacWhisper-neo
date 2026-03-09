pub mod anthropic;
pub mod whisper_cpp;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::segment::Segment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub language: String,
    pub duration_ms: i64,
    pub segments: Vec<Segment>,
    pub full_text: String,
}

#[async_trait]
pub trait TranscriptionEngine: Send + Sync {
    fn engine_id(&self) -> &str;
    async fn transcribe(
        &self,
        audio_path: &str,
        language: Option<&str>,
    ) -> Result<TranscriptionResult, AppError>;
}

pub fn create_engine(
    engine_id: &str,
    api_key: &str,
) -> Result<Box<dyn TranscriptionEngine>, AppError> {
    match engine_id {
        "anthropic" => Ok(Box::new(anthropic::AnthropicEngine::new(
            api_key.to_string(),
        ))),
        "whisper_cpp" => Err(AppError::NotImplemented(
            "whisper_cpp engine is not yet available".into(),
        )),
        _ => Err(AppError::InvalidEngine(engine_id.to_string())),
    }
}
