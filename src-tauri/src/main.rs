mod commands;
mod db;
mod engine;
mod error;
mod models;

use db::Database;

fn main() {
    let database = Database::open(None).expect("failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(database)
        .invoke_handler(tauri::generate_handler![
            commands::transcribe::run_transcription,
            commands::history::list_transcripts,
            commands::history::get_transcript,
            commands::file::resolve_dropped_file,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
