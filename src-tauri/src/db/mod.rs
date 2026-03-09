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
