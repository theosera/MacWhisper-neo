use std::sync::Mutex;

use crate::db::Database;
use crate::engine::{ModelInfo, ProviderInfo, ProviderRegistry};

#[tauri::command]
pub fn list_providers(
    registry: tauri::State<'_, Mutex<ProviderRegistry>>,
    db: tauri::State<'_, Database>,
) -> Vec<ProviderInfo> {
    let reg = registry.lock().unwrap();
    let mut providers = reg.list_providers();

    // カスタムモデルをマージ
    for provider in &mut providers {
        if let Ok(custom_models) = db.list_custom_models(&provider.id) {
            for (model_id, name, description, max_file_size_mb) in custom_models {
                provider.models.push(ModelInfo {
                    id: model_id,
                    name,
                    description,
                    max_file_size_mb,
                });
            }
        }
    }

    providers
}
