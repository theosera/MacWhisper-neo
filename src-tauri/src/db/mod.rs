pub mod schema;

use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::AppError;
use crate::models::segment::Segment;
use crate::models::transcript::{Transcript, TranscriptSummary};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(db_path: Option<&str>) -> Result<Self, AppError> {
        let path = Self::resolve_path(db_path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        for migration in schema::MIGRATIONS {
            conn.execute_batch(migration)?;
        }

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn resolve_path(override_path: Option<&str>) -> Result<PathBuf, AppError> {
        if let Some(p) = override_path {
            return Ok(PathBuf::from(p));
        }
        if let Ok(p) = std::env::var("MACWHISPER_DB_PATH") {
            return Ok(PathBuf::from(p));
        }
        let cwd = std::env::current_dir()?;
        let root = if cwd.ends_with("src-tauri") {
            cwd.parent().map(|p| p.to_path_buf()).unwrap_or(cwd)
        } else {
            cwd
        };
        Ok(root.join("data").join("app.db"))
    }

    pub fn insert_transcript(
        &self,
        id: &str,
        created_at: &str,
        audio_path: &str,
        file_name: &str,
        engine_id: &str,
        language: &str,
        duration_ms: i64,
        processing_time_ms: i64,
        full_text: &str,
    ) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO transcripts
               (id, created_at, audio_path, file_name, engine_id, language, duration_ms, processing_time_ms, full_text)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            params![id, created_at, audio_path, file_name, engine_id, language, duration_ms, processing_time_ms, full_text],
        )?;
        Ok(())
    }

    pub fn insert_segments(&self, segments: &[Segment]) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"INSERT INTO segments (id, transcript_id, start_ms, end_ms, text)
               VALUES (?1, ?2, ?3, ?4, ?5)"#,
        )?;
        for seg in segments {
            stmt.execute(params![seg.id, seg.transcript_id, seg.start_ms, seg.end_ms, seg.text])?;
        }
        Ok(())
    }

    pub fn list_transcripts(&self, limit: i64) -> Result<Vec<TranscriptSummary>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, created_at, file_name, engine_id, language, full_text
               FROM transcripts ORDER BY created_at DESC LIMIT ?1"#,
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            let full_text: String = row.get(5)?;
            let preview = full_text.chars().take(100).collect::<String>();
            Ok(TranscriptSummary {
                id: row.get(0)?,
                created_at: row.get(1)?,
                file_name: row.get(2)?,
                engine_id: row.get(3)?,
                language: row.get(4)?,
                preview,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn add_custom_model(
        &self,
        provider_id: &str,
        model_id: &str,
        name: &str,
        description: &str,
        max_file_size_mb: u32,
    ) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();
        conn.execute(
            r#"INSERT INTO custom_models (id, provider_id, model_id, name, description, max_file_size_mb, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
               ON CONFLICT(provider_id, model_id) DO UPDATE SET
                   name = excluded.name,
                   description = excluded.description,
                   max_file_size_mb = excluded.max_file_size_mb"#,
            params![id, provider_id, model_id, name, description, max_file_size_mb as i64, created_at],
        )?;
        Ok(())
    }

    pub fn list_custom_models(&self, provider_id: &str) -> Result<Vec<(String, String, String, u32)>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT model_id, name, description, max_file_size_mb FROM custom_models WHERE provider_id = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![provider_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)? as u32,
            ))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn delete_custom_model(&self, provider_id: &str, model_id: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM custom_models WHERE provider_id = ?1 AND model_id = ?2",
            params![provider_id, model_id],
        )?;
        Ok(())
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        for migration in schema::MIGRATIONS {
            conn.execute_batch(migration)?;
        }
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn get_transcript(&self, id: &str) -> Result<Transcript, AppError> {
        let conn = self.conn.lock().unwrap();
        let transcript = conn.query_row(
            r#"SELECT id, created_at, audio_path, file_name, engine_id, language,
                      duration_ms, processing_time_ms, full_text
               FROM transcripts WHERE id = ?1"#,
            params![id],
            |row| {
                Ok(Transcript {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    audio_path: row.get(2)?,
                    file_name: row.get(3)?,
                    engine_id: row.get(4)?,
                    language: row.get(5)?,
                    duration_ms: row.get(6)?,
                    processing_time_ms: row.get(7)?,
                    full_text: row.get(8)?,
                    segments: Vec::new(),
                })
            },
        )?;

        let mut stmt = conn.prepare(
            r#"SELECT id, transcript_id, start_ms, end_ms, text
               FROM segments WHERE transcript_id = ?1 ORDER BY start_ms"#,
        )?;
        let segments: Vec<Segment> = stmt
            .query_map(params![id], |row| {
                Ok(Segment {
                    id: row.get(0)?,
                    transcript_id: row.get(1)?,
                    start_ms: row.get(2)?,
                    end_ms: row.get(3)?,
                    text: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Transcript {
            segments,
            ..transcript
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db() -> Database {
        Database::open_in_memory().expect("in-memory DB should open")
    }

    #[test]
    fn test_settings_roundtrip() {
        let db = make_db();
        assert_eq!(db.get_setting("missing_key").unwrap(), None);

        db.set_setting("api_key", "sk-test-123").unwrap();
        assert_eq!(
            db.get_setting("api_key").unwrap(),
            Some("sk-test-123".to_string())
        );

        // 更新
        db.set_setting("api_key", "sk-updated").unwrap();
        assert_eq!(
            db.get_setting("api_key").unwrap(),
            Some("sk-updated".to_string())
        );
    }

    #[test]
    fn test_insert_and_list_transcript() {
        let db = make_db();

        db.insert_transcript(
            "tr-001",
            "2026-01-01T00:00:00Z",
            "/tmp/test.mp3",
            "test.mp3",
            "anthropic/claude-sonnet-4",
            "ja",
            60_000,
            1_500,
            "テスト文字起こし",
        )
        .unwrap();

        let list = db.list_transcripts(10).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "tr-001");
        assert_eq!(list[0].file_name, "test.mp3");
        assert_eq!(list[0].language, "ja");
        assert!(list[0].preview.contains("テスト"));
    }

    #[test]
    fn test_insert_segments_and_get_transcript() {
        let db = make_db();

        db.insert_transcript(
            "tr-002",
            "2026-01-01T00:00:00Z",
            "/tmp/audio.wav",
            "audio.wav",
            "openai_whisper/whisper-1",
            "en",
            30_000,
            800,
            "Hello world. This is a test.",
        )
        .unwrap();

        let segments = vec![
            Segment {
                id: "seg-001".to_string(),
                transcript_id: "tr-002".to_string(),
                start_ms: 0,
                end_ms: 2000,
                text: "Hello world.".to_string(),
            },
            Segment {
                id: "seg-002".to_string(),
                transcript_id: "tr-002".to_string(),
                start_ms: 2000,
                end_ms: 5000,
                text: "This is a test.".to_string(),
            },
        ];
        db.insert_segments(&segments).unwrap();

        let transcript = db.get_transcript("tr-002").unwrap();
        assert_eq!(transcript.segments.len(), 2);
        assert_eq!(transcript.segments[0].start_ms, 0);
        assert_eq!(transcript.segments[1].text, "This is a test.");
    }

    #[test]
    fn test_list_transcripts_order() {
        let db = make_db();

        db.insert_transcript("tr-old", "2026-01-01T00:00:00Z", "/a.mp3", "a.mp3",
            "anthropic", "en", 0, 0, "old").unwrap();
        db.insert_transcript("tr-new", "2026-06-01T00:00:00Z", "/b.mp3", "b.mp3",
            "anthropic", "en", 0, 0, "new").unwrap();

        let list = db.list_transcripts(10).unwrap();
        // 新しい順に並ぶこと
        assert_eq!(list[0].id, "tr-new");
        assert_eq!(list[1].id, "tr-old");
    }
}
