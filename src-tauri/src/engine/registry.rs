use std::collections::HashMap;
use super::{TranscriptionProvider, ProviderInfo};
use crate::error::AppError;

pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn TranscriptionProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn TranscriptionProvider>) {
        self.providers.insert(provider.provider_id().to_string(), provider);
    }

    pub fn get(&self, provider_id: &str) -> Option<&dyn TranscriptionProvider> {
        self.providers.get(provider_id).map(|p| p.as_ref())
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

    pub fn get_provider_mut(&mut self, provider_id: &str) -> Option<&mut dyn TranscriptionProvider> {
        self.providers.get_mut(provider_id).map(|p| p.as_mut())
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
