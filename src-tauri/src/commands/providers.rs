use std::sync::Mutex;

use crate::engine::{ProviderInfo, ProviderRegistry};

#[tauri::command]
pub fn list_providers(
    registry: tauri::State<'_, Mutex<ProviderRegistry>>,
) -> Vec<ProviderInfo> {
    let reg = registry.lock().unwrap();
    reg.list_providers()
}
