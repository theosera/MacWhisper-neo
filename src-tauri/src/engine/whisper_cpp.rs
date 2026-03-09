use async_trait::async_trait;

use super::{TranscriptionEngine, TranscriptionResult};
use crate::error::AppError;

pub struct WhisperCppEngine;

#[async_trait]
impl TranscriptionEngine for WhisperCppEngine {
    fn engine_id(&self) -> &str {
        "whisper_cpp"
    }

    async fn transcribe(
        &self,
        _audio_path: &str,
        _language: Option<&str>,
    ) -> Result<TranscriptionResult, AppError> {
        Err(AppError::NotImplemented(
            "whisper.cpp engine will be implemented in a future release".into(),
        ))
    }
}
