pub const MIGRATIONS: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS transcripts (
        id TEXT PRIMARY KEY,
        created_at TEXT NOT NULL,
        audio_path TEXT NOT NULL,
        file_name TEXT NOT NULL,
        engine_id TEXT NOT NULL,
        language TEXT NOT NULL DEFAULT 'auto',
        duration_ms INTEGER NOT NULL DEFAULT 0,
        processing_time_ms INTEGER NOT NULL DEFAULT 0,
        full_text TEXT NOT NULL
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS segments (
        id TEXT PRIMARY KEY,
        transcript_id TEXT NOT NULL,
        start_ms INTEGER NOT NULL,
        end_ms INTEGER NOT NULL,
        text TEXT NOT NULL,
        FOREIGN KEY(transcript_id) REFERENCES transcripts(id) ON DELETE CASCADE
    );
    "#,
    r#"
    CREATE INDEX IF NOT EXISTS idx_segments_transcript_id ON segments(transcript_id);
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
    "#,
];
