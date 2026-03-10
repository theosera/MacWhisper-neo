use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::error::AppError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCustomModelRequest {
    pub provider_id: String,
    pub model_id: String,
    pub name: String,
    pub description: String,
    pub max_file_size_mb: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomModelInfo {
    pub model_id: String,
    pub name: String,
    pub description: String,
    pub max_file_size_mb: u32,
}

#[tauri::command]
pub fn add_custom_model(
    request: AddCustomModelRequest,
    db: tauri::State<'_, Database>,
) -> Result<(), AppError> {
    db.add_custom_model(
        &request.provider_id,
        &request.model_id,
        &request.name,
        &request.description,
        request.max_file_size_mb,
    )
}

#[tauri::command]
pub fn list_custom_models(
    provider_id: String,
    db: tauri::State<'_, Database>,
) -> Result<Vec<CustomModelInfo>, AppError> {
    let models = db.list_custom_models(&provider_id)?;
    Ok(models
        .into_iter()
        .map(|(model_id, name, description, max_file_size_mb)| CustomModelInfo {
            model_id,
            name,
            description,
            max_file_size_mb,
        })
        .collect())
}

#[tauri::command]
pub fn delete_custom_model(
    provider_id: String,
    model_id: String,
    db: tauri::State<'_, Database>,
) -> Result<(), AppError> {
    db.delete_custom_model(&provider_id, &model_id)
}
