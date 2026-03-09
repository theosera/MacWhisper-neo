use async_trait::async_trait;

use super::{TranscriptionProvider, TranscriptionResult, ModelInfo, ProviderCategory};
use crate::error::AppError;

pub struct WhisperCppProvider;

#[async_trait]
impl TranscriptionProvider for WhisperCppProvider {
    fn provider_id(&self) -> &str {
        "whisper_cpp"
    }

    fn provider_name(&self) -> &str {
        "whisper.cpp (Local)"
    }

    fn category(&self) -> ProviderCategory {
        ProviderCategory::LocalBinary
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: "base".to_string(),
            name: "Base".to_string(),
            description: "Not yet implemented".to_string(),
            max_file_size_mb: 0,
        }]
    }

    async fn transcribe(
        &self,
        _audio_path: &str,
        _model_id: &str,
        _language: Option<&str>,
    ) -> Result<TranscriptionResult, AppError> {
        Err(AppError::NotImplemented(
            "whisper.cpp engine will be implemented in a future release".into(),
        ))
    }
}
