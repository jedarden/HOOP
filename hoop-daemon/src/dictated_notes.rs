//! Dictated Notes service
//!
//! Manages the lifecycle of dictated notes — Stitches with `kind='dictated'`.
//! Each note has audio + transcript stored as attachments under
//! `~/.hoop/attachments/<stitch-id>/`, with metadata in the `dictated_notes` table.
//!
//! Flow:
//! 1. Audio captured (hotkey / ADB / upload)
//! 2. Note created with `transcription_status: Pending`
//! 3. Whisper job submitted to async queue
//! 4. On success: transcript + word timestamps stored, status → Completed
//! 5. On failure: partial transcript saved with error, status → Failed (UI shows warning card)
//! 6. Appears in project stitch list by `last_activity_at`

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Transcription status for a dictated note
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TranscriptionStatus {
    /// Waiting in queue or currently transcribing
    Pending,
    /// Transcription completed successfully
    Completed,
    /// Transcription failed; partial transcript may be available
    Failed,
}

/// A dictated note with all metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictatedNote {
    /// FK to the parent stitch (kind=dictated)
    pub stitch_id: String,
    /// When the audio was captured
    pub recorded_at: DateTime<Utc>,
    /// When Whisper transcription completed
    pub transcribed_at: DateTime<Utc>,
    /// Filename of the audio attachment
    pub audio_filename: String,
    /// Full text transcript
    pub transcript: String,
    /// Word-level timestamps from Whisper
    pub transcript_words: Vec<TranscriptWord>,
    /// Audio duration in seconds
    pub duration_secs: Option<f64>,
    /// Detected language code
    pub language: Option<String>,
    /// Optional tags
    pub tags: Vec<String>,
    /// Whether transcription is pending, completed, or failed
    pub transcription_status: TranscriptionStatus,
}

/// A single word with timing from Whisper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptWord {
    pub word: String,
    pub start: f64,
    pub end: f64,
}

/// Request to create a new dictated note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteRequest {
    /// Project to create the note in
    pub project: String,
    /// Raw audio bytes (base64-encoded for JSON transport)
    pub audio_data: String,
    /// Audio filename (e.g. "note-2026-04-22t15-30.webm")
    pub audio_filename: String,
    /// MIME type of the audio
    pub audio_content_type: String,
    /// Optional pre-computed transcript (if Whisper already ran client-side)
    pub transcript: Option<String>,
    /// Optional word-level timestamps
    pub transcript_words: Option<Vec<TranscriptWord>>,
    /// Optional audio duration
    pub duration_secs: Option<f64>,
    /// Optional detected language
    pub language: Option<String>,
    /// Optional tags
    pub tags: Option<Vec<String>>,
}

/// Response after creating a dictated note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteResponse {
    pub stitch_id: String,
    pub project: String,
    pub title: String,
    pub recorded_at: DateTime<Utc>,
    pub transcribed_at: DateTime<Utc>,
    pub transcription_status: TranscriptionStatus,
}

/// Summary of a dictated note for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSummary {
    pub stitch_id: String,
    pub project: String,
    pub title: String,
    pub kind: String,
    pub recorded_at: DateTime<Utc>,
    pub transcribed_at: DateTime<Utc>,
    pub duration_secs: Option<f64>,
    pub language: Option<String>,
    pub tags: Vec<String>,
    pub transcript_preview: String,
    /// Full transcript text for search indexing
    pub transcript: String,
    pub last_activity_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub audio_filename: String,
    pub transcription_status: TranscriptionStatus,
}

/// Derive a title from a transcript (first meaningful line, truncated)
pub fn derive_title(transcript: &str) -> String {
    let line = transcript
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())
        .unwrap_or("Voice note");
    if line.len() > 80 {
        format!("{}…", &line[..79])
    } else {
        line.to_string()
    }
}

/// Truncate transcript for preview display
pub fn transcript_preview(transcript: &str, max_len: usize) -> String {
    if transcript.len() <= max_len {
        transcript.to_string()
    } else {
        let end = transcript[..max_len]
            .rfind(' ')
            .unwrap_or(max_len);
        format!("{}…", &transcript[..end])
    }
}

/// Persist audio data to the stitch attachments directory
pub fn store_audio(
    stitch_id: &str,
    audio_filename: &str,
    audio_data: &[u8],
) -> Result<PathBuf> {
    let dir = stitch_attachment_dir(stitch_id)?;
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create attachment dir: {}", dir.display()))?;

    let path = dir.join(audio_filename);
    // Atomic write via .tmp + rename
    let tmp_path = path.with_extension("tmp");
    std::fs::write(&tmp_path, audio_data)
        .with_context(|| format!("Failed to write audio tmp: {}", tmp_path.display()))?;
    std::fs::rename(&tmp_path, &path)
        .with_context(|| format!("Failed to rename audio: {}", path.display()))?;

    Ok(path)
}

/// Get the attachment directory for a stitch
pub fn stitch_attachment_dir(stitch_id: &str) -> Result<PathBuf> {
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.push(".hoop");
    home.push("attachments");
    home.push(stitch_id);
    Ok(home)
}

/// Get the full path to an audio file for a stitch
pub fn audio_path(stitch_id: &str, audio_filename: &str) -> Result<PathBuf> {
    if audio_filename.contains('/') || audio_filename.contains('\\') || audio_filename.contains("..") {
        anyhow::bail!("Invalid audio filename: {}", audio_filename);
    }
    Ok(stitch_attachment_dir(stitch_id)?.join(audio_filename))
}

/// Insert a dictated note into the database
pub fn insert_note(conn: &Connection, note: &DictatedNote) -> Result<()> {
    let words_json = serde_json::to_string(&note.transcript_words)
        .context("Failed to serialize transcript_words")?;
    let tags_json = serde_json::to_string(&note.tags)
        .context("Failed to serialize tags")?;
    let status_str = serde_json::to_string(&note.transcription_status)
        .context("Failed to serialize transcription_status")?;

    conn.execute(
        r#"
        INSERT INTO dictated_notes (stitch_id, recorded_at, transcribed_at, audio_filename,
                                     transcript, transcript_words, duration_secs, language, tags,
                                     transcription_status)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
        params![
            note.stitch_id,
            note.recorded_at.to_rfc3339(),
            note.transcribed_at.to_rfc3339(),
            note.audio_filename,
            note.transcript,
            words_json,
            note.duration_secs,
            note.language,
            tags_json,
            status_str,
        ],
    )
    .context("Failed to insert dictated_note")?;

    Ok(())
}

/// Insert the stitch row for a dictated note
pub fn insert_stitch(
    conn: &Connection,
    stitch_id: &str,
    project: &str,
    title: &str,
    created_by: &str,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        r#"
        INSERT INTO stitches (id, project, kind, title, created_by, created_at, last_activity_at,
                              participants, attachments_path)
        VALUES (?1, ?2, 'dictated', ?3, ?4, ?5, ?5, '[]', ?6)
        "#,
        params![
            stitch_id,
            project,
            title,
            created_by,
            now,
            format!("~/.hoop/attachments/{}", stitch_id),
        ],
    )
    .context("Failed to insert stitch for dictated note")?;

    Ok(())
}

/// Parse transcription status from DB string
fn parse_transcription_status(s: &str) -> TranscriptionStatus {
    match s.trim_matches('"') {
        "Completed" => TranscriptionStatus::Completed,
        "Failed" => TranscriptionStatus::Failed,
        _ => TranscriptionStatus::Pending,
    }
}

/// Get a single dictated note by stitch_id
pub fn get_note(conn: &Connection, stitch_id: &str) -> Result<Option<DictatedNote>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT stitch_id, recorded_at, transcribed_at, audio_filename,
               transcript, transcript_words, duration_secs, language, tags,
               COALESCE(transcription_status, '"Pending"')
        FROM dictated_notes
        WHERE stitch_id = ?1
        "#,
    )?;

    let result = stmt.query_row(params![stitch_id], |row| {
        let recorded_at_str: String = row.get(1)?;
        let transcribed_at_str: String = row.get(2)?;
        let audio_filename: String = row.get(3)?;
        let transcript: String = row.get(4)?;
        let words_json: Option<String> = row.get(5)?;
        let duration_secs: Option<f64> = row.get(6)?;
        let language: Option<String> = row.get(7)?;
        let tags_json: String = row.get(8)?;
        let status_str: String = row.get(9)?;

        let recorded_at = DateTime::parse_from_rfc3339(&recorded_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
        let transcribed_at = DateTime::parse_from_rfc3339(&transcribed_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;

        let transcript_words: Vec<TranscriptWord> = words_json
            .and_then(|j| serde_json::from_str(&j).ok())
            .unwrap_or_default();
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        let transcription_status = parse_transcription_status(&status_str);

        Ok(DictatedNote {
            stitch_id: stitch_id.to_string(),
            recorded_at,
            transcribed_at,
            audio_filename,
            transcript,
            transcript_words,
            duration_secs,
            language,
            tags,
            transcription_status,
        })
    });

    match result {
        Ok(note) => Ok(Some(note)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e).context("Failed to query dictated_note"),
    }
}

/// List dictated notes for a project, ordered by last_activity_at DESC
pub fn list_notes_for_project(conn: &Connection, project: &str) -> Result<Vec<NoteSummary>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT dn.stitch_id, s.project, s.title, dn.recorded_at, dn.transcribed_at,
               dn.duration_secs, dn.language, dn.tags, dn.transcript, s.last_activity_at,
               s.created_at, dn.audio_filename,
               COALESCE(dn.transcription_status, '"Pending"')
        FROM dictated_notes dn
        JOIN stitches s ON dn.stitch_id = s.id
        WHERE s.project = ?1
        ORDER BY s.last_activity_at DESC
        "#,
    )?;

    let notes = stmt
        .query_map(params![project], |row| {
            let stitch_id: String = row.get(0)?;
            let project: String = row.get(1)?;
            let title: String = row.get(2)?;
            let recorded_at_str: String = row.get(3)?;
            let transcribed_at_str: String = row.get(4)?;
            let duration_secs: Option<f64> = row.get(5)?;
            let language: Option<String> = row.get(6)?;
            let tags_json: String = row.get(7)?;
            let transcript: String = row.get(8)?;
            let last_activity_at_str: String = row.get(9)?;
            let created_at_str: String = row.get(10)?;
            let audio_filename: String = row.get(11)?;
            let status_str: String = row.get(12)?;

            let recorded_at = DateTime::parse_from_rfc3339(&recorded_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
            let transcribed_at = DateTime::parse_from_rfc3339(&transcribed_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
            let last_activity_at = DateTime::parse_from_rfc3339(&last_activity_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;

            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            let preview = transcript_preview(&transcript, 120);
            let transcription_status = parse_transcription_status(&status_str);

            Ok(NoteSummary {
                stitch_id,
                project,
                title,
                kind: "dictated".to_string(),
                recorded_at,
                transcribed_at,
                duration_secs,
                language,
                tags,
                transcript_preview: preview,
                transcript,
                last_activity_at,
                created_at,
                audio_filename,
                transcription_status,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(notes)
}

/// Update a dictated note's transcript and/or tags
pub fn update_note(conn: &Connection, note: &DictatedNote) -> Result<()> {
    let words_json = serde_json::to_string(&note.transcript_words)
        .context("Failed to serialize transcript_words")?;
    let tags_json = serde_json::to_string(&note.tags)
        .context("Failed to serialize tags")?;
    let status_str = serde_json::to_string(&note.transcription_status)
        .context("Failed to serialize transcription_status")?;

    conn.execute(
        r#"
        UPDATE dictated_notes
        SET transcript = ?1, transcript_words = ?2, tags = ?3, transcription_status = ?4
        WHERE stitch_id = ?5
        "#,
        params![
            note.transcript,
            words_json,
            tags_json,
            status_str,
            note.stitch_id,
        ],
    )
    .context("Failed to update dictated_note")?;

    Ok(())
}

/// Update the title on a dictated note's stitch row
pub fn update_stitch_title(conn: &Connection, stitch_id: &str, title: &str) -> Result<()> {
    conn.execute(
        "UPDATE stitches SET title = ?1 WHERE id = ?2",
        params![title, stitch_id],
    )
    .context("Failed to update stitch title")?;
    Ok(())
}

/// List all dictated notes across all projects, ordered by last_activity_at DESC
pub fn list_all_notes(conn: &Connection) -> Result<Vec<NoteSummary>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT dn.stitch_id, s.project, s.title, dn.recorded_at, dn.transcribed_at,
               dn.duration_secs, dn.language, dn.tags, dn.transcript, s.last_activity_at,
               s.created_at, dn.audio_filename,
               COALESCE(dn.transcription_status, '"Pending"')
        FROM dictated_notes dn
        JOIN stitches s ON dn.stitch_id = s.id
        ORDER BY s.last_activity_at DESC
        "#,
    )?;

    let notes = stmt
        .query_map([], |row| {
            let stitch_id: String = row.get(0)?;
            let project: String = row.get(1)?;
            let title: String = row.get(2)?;
            let recorded_at_str: String = row.get(3)?;
            let transcribed_at_str: String = row.get(4)?;
            let duration_secs: Option<f64> = row.get(5)?;
            let language: Option<String> = row.get(6)?;
            let tags_json: String = row.get(7)?;
            let transcript: String = row.get(8)?;
            let last_activity_at_str: String = row.get(9)?;
            let created_at_str: String = row.get(10)?;
            let audio_filename: String = row.get(11)?;
            let status_str: String = row.get(12)?;

            let recorded_at = DateTime::parse_from_rfc3339(&recorded_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
            let transcribed_at = DateTime::parse_from_rfc3339(&transcribed_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
            let last_activity_at = DateTime::parse_from_rfc3339(&last_activity_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;

            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            let preview = transcript_preview(&transcript, 120);
            let transcription_status = parse_transcription_status(&status_str);

            Ok(NoteSummary {
                stitch_id,
                project,
                title,
                kind: "dictated".to_string(),
                recorded_at,
                transcribed_at,
                duration_secs,
                language,
                tags,
                transcript_preview: preview,
                transcript,
                last_activity_at,
                created_at,
                audio_filename,
                transcription_status,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(notes)
}

/// Result from Whisper transcription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub transcript: String,
    pub words: Vec<TranscriptWord>,
    pub duration_secs: Option<f64>,
    pub language: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_title_short() {
        let transcript = "Hello world\nSecond line";
        assert_eq!(derive_title(transcript), "Hello world");
    }

    #[test]
    fn test_derive_title_long() {
        let long_line = "This is a very long line that exceeds the eighty character limit and should be truncated with an ellipsis at the end";
        let title = derive_title(long_line);
        assert!(title.len() <= 81);
        assert!(title.ends_with('…'));
    }

    #[test]
    fn test_derive_title_empty() {
        assert_eq!(derive_title(""), "Voice note");
        assert_eq!(derive_title("   \n  \n"), "Voice note");
    }

    #[test]
    fn test_derive_title_whitespace_lines() {
        assert_eq!(derive_title("\n\n\nactual content"), "actual content");
    }

    #[test]
    fn test_transcript_preview_short() {
        assert_eq!(transcript_preview("hello", 100), "hello");
    }

    #[test]
    fn test_transcript_preview_truncates_at_word() {
        let text = "The quick brown fox jumps over the lazy dog and continues";
        let preview = transcript_preview(text, 30);
        assert!(preview.ends_with('…'));
        assert!(!preview.contains("lazy"));
    }

    #[test]
    fn test_audio_path_rejects_traversal() {
        assert!(audio_path("abc", "../etc/passwd").is_err());
        assert!(audio_path("abc", "sub/file.wav").is_err());
        assert!(audio_path("abc", "file.wav").is_ok());
    }

    #[test]
    fn test_insert_and_get_note() {
        let conn = init_test_db();

        let note = DictatedNote {
            stitch_id: "st-001".to_string(),
            recorded_at: Utc::now(),
            transcribed_at: Utc::now(),
            audio_filename: "note.webm".to_string(),
            transcript: "Test transcript".to_string(),
            transcript_words: vec![TranscriptWord {
                word: "Test".to_string(),
                start: 0.0,
                end: 0.5,
            }],
            duration_secs: Some(3.0),
            language: Some("en".to_string()),
            tags: vec!["test".to_string()],
            transcription_status: TranscriptionStatus::Completed,
        };

        insert_note(&conn, &note).unwrap();

        let fetched = get_note(&conn, "st-001").unwrap().unwrap();
        assert_eq!(fetched.stitch_id, "st-001");
        assert_eq!(fetched.transcript, "Test transcript");
        assert_eq!(fetched.audio_filename, "note.webm");
        assert_eq!(fetched.transcript_words.len(), 1);
        assert_eq!(fetched.tags, vec!["test"]);
        assert_eq!(fetched.duration_secs, Some(3.0));
        assert_eq!(fetched.language, Some("en".to_string()));
        assert_eq!(fetched.transcription_status, TranscriptionStatus::Completed);
    }

    #[test]
    fn test_get_note_nonexistent() {
        let conn = init_test_db();
        assert!(get_note(&conn, "nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_list_notes_for_project() {
        let conn = init_test_db();

        for (id, project, title) in [
            ("st-001", "project-a", "Note one"),
            ("st-002", "project-a", "Note two"),
            ("st-003", "project-b", "Note three"),
        ] {
            insert_stitch(&conn, id, project, title, "operator").unwrap();
            insert_note(
                &conn,
                &DictatedNote {
                    stitch_id: id.to_string(),
                    recorded_at: Utc::now(),
                    transcribed_at: Utc::now(),
                    audio_filename: format!("{}.webm", id),
                    transcript: format!("Transcript for {}", id),
                    transcript_words: vec![],
                    duration_secs: None,
                    language: None,
                    tags: vec![],
                    transcription_status: TranscriptionStatus::Pending,
                },
            )
            .unwrap();
        }

        let project_a = list_notes_for_project(&conn, "project-a").unwrap();
        assert_eq!(project_a.len(), 2);
        assert!(project_a.iter().all(|n| n.project == "project-a"));
        assert!(project_a.iter().all(|n| n.transcription_status == TranscriptionStatus::Pending));

        let project_b = list_notes_for_project(&conn, "project-b").unwrap();
        assert_eq!(project_b.len(), 1);
        assert_eq!(project_b[0].title, "Note three");
    }

    fn init_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();

        conn.execute(
            r#"
            CREATE TABLE stitches (
                id TEXT PRIMARY KEY NOT NULL,
                project TEXT NOT NULL,
                kind TEXT NOT NULL CHECK(kind IN ('operator', 'dictated', 'worker', 'ad-hoc')),
                title TEXT NOT NULL,
                created_by TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_activity_at TEXT NOT NULL,
                participants TEXT DEFAULT '[]',
                attachments_path TEXT
            )
            "#,
            [],
        )
        .unwrap();

        conn.execute(
            r#"
            CREATE TABLE dictated_notes (
                stitch_id TEXT PRIMARY KEY NOT NULL REFERENCES stitches(id) ON DELETE CASCADE,
                recorded_at TEXT NOT NULL,
                transcribed_at TEXT NOT NULL,
                audio_filename TEXT NOT NULL,
                transcript TEXT NOT NULL,
                duration_secs REAL,
                language TEXT,
                tags TEXT DEFAULT '[]',
                transcript_words TEXT,
                transcription_status TEXT NOT NULL DEFAULT '"Pending"'
            )
            "#,
            [],
        )
        .unwrap();

        conn
    }
}
