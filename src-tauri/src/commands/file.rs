use crate::error::AppError;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub path: String,
    pub file_name: String,
    pub extension: String,
    pub size_bytes: u64,
}

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "m4a", "aac", "ogg", "oga", "flac", "webm", "mp4", "mov", "mkv", "avi",
];

#[tauri::command]
pub fn resolve_dropped_file(path: String) -> Result<FileInfo, AppError> {
    let p = Path::new(&path);
    if !p.is_file() {
        return Err(AppError::FileNotFound(path));
    }

    let extension = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
        return Err(AppError::Validation(format!(
            "Unsupported file format: .{extension}. Supported: {}",
            SUPPORTED_EXTENSIONS.join(", ")
        )));
    }

    let metadata = std::fs::metadata(p)?;
    let file_name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(FileInfo {
        path,
        file_name,
        extension,
        size_bytes: metadata.len(),
    })
}
