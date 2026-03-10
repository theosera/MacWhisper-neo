mod commands;
mod db;
mod engine;
mod error;
mod models;

use std::sync::Mutex;

use db::Database;
use engine::{create_default_registry, ProviderConfig};

fn main() {
    let database = Database::open(None).expect("failed to open database");

    let config = ProviderConfig {
        anthropic_api_key: String::new(),
        openai_api_key: String::new(),
        google_gemini_api_key: String::new(),
        lm_studio_endpoint: "http://localhost:1234".to_string(),
    };
    let registry = Mutex::new(create_default_registry(&config));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(database)
        .manage(registry)
        .invoke_handler(tauri::generate_handler![
            commands::transcribe::run_transcription,
            commands::history::list_transcripts,
            commands::history::get_transcript,
            commands::file::resolve_dropped_file,
            commands::providers::list_providers,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::youtube::download_youtube,
            commands::youtube::cleanup_youtube_temp_file,
            commands::models::add_custom_model,
            commands::models::list_custom_models,
            commands::models::delete_custom_model,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
