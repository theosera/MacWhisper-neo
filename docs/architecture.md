# Architecture (MVP)

## Runtime Layers
1. **UI**: React 19 + Vite + TypeScript + Zustand
2. **Bridge**: Tauri 2 commands (async)
3. **Core**: Rust — engine / db / models / error
4. **External**: Anthropic API (MVP), whisper.cpp (future)

## Engine Trait
`TranscriptionEngine` trait in `src-tauri/src/engine/mod.rs`:
- `engine_id()` — returns identifier string
- `transcribe(audio_path, language)` — async, returns `TranscriptionResult`
- Factory function `create_engine(engine_id, api_key)` resolves implementation

## Data Model (SQLite)
- `transcripts` — id, created_at, audio_path, file_name, engine_id, language, duration_ms, processing_time_ms, full_text
- `segments` — id, transcript_id, start_ms, end_ms, text (FK to transcripts)
- `settings` — key/value store for configuration

## UI Layout
3-pane grid: History (240px) | Transcript (flex) | Meta (260px)

## Export
Frontend generates Markdown / TXT / JSON from Transcript data,
saves via Tauri dialog + fs plugins.
