pub mod anthropic;
pub mod gemini;
pub mod lm_studio;
pub mod openai_whisper;
pub mod registry;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub max_file_size_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCategory {
    ApiCloud,
    ApiLocal,
    LocalBinary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub category: ProviderCategory,
    pub models: Vec<ModelInfo>,
}

#[async_trait]
pub trait TranscriptionProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    fn provider_name(&self) -> &str;
    fn category(&self) -> ProviderCategory;
    fn available_models(&self) -> Vec<ModelInfo>;
    async fn transcribe(
        &self,
        audio_path: &str,
        model_id: &str,
        language: Option<&str>,
    ) -> Result<TranscriptionResult, AppError>;
}

pub use registry::ProviderRegistry;

pub fn create_default_registry(config: &ProviderConfig) -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(anthropic::AnthropicProvider::new(
        config.anthropic_api_key.clone(),
    )));
    registry.register(Box::new(openai_whisper::OpenAIWhisperProvider::new(
        config.openai_api_key.clone(),
    )));
    registry.register(Box::new(gemini::GeminiProvider::new(
        config.google_gemini_api_key.clone(),
    )));
    registry.register(Box::new(lm_studio::LmStudioProvider::new(
        config.lm_studio_endpoint.clone(),
    )));
    registry
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub anthropic_api_key: String,
    pub openai_api_key: String,
    pub google_gemini_api_key: String,
    pub lm_studio_endpoint: String,
}
