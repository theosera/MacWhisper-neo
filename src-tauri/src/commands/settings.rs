use crate::db::Database;
use crate::error::AppError;

/// 設定値を取得する。存在しない場合は None を返す
#[tauri::command]
pub fn get_setting(key: String, db: tauri::State<'_, Database>) -> Result<Option<String>, AppError> {
    db.get_setting(&key)
}

/// 設定値を保存/更新する
#[tauri::command]
pub fn set_setting(key: String, value: String, db: tauri::State<'_, Database>) -> Result<(), AppError> {
    db.set_setting(&key, &value)
}
