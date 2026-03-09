use std::collections::HashMap;
use std::sync::Arc;
use super::{TranscriptionProvider, TranscriptionResult, ProviderInfo};
use crate::error::AppError;

pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn TranscriptionProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn TranscriptionProvider>) {
        self.providers.insert(provider.provider_id().to_string(), Arc::from(provider));
    }

    pub fn get(&self, provider_id: &str) -> Option<Arc<dyn TranscriptionProvider>> {
        self.providers.get(provider_id).cloned()
    }

    pub fn list_providers(&self) -> Vec<ProviderInfo> {
        self.providers
            .values()
            .map(|p| ProviderInfo {
                id: p.provider_id().to_string(),
                name: p.provider_name().to_string(),
                category: p.category(),
                models: p.available_models(),
            })
            .collect()
    }

    pub async fn transcribe(
        &self,
        provider_id: &str,
        audio_path: &str,
        model_id: &str,
        language: Option<&str>,
    ) -> Result<TranscriptionResult, AppError> {
        let provider = self
            .get(provider_id)
            .ok_or_else(|| AppError::InvalidEngine(provider_id.to_string()))?;
        provider.transcribe(audio_path, model_id, language).await
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
