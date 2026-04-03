use chrono::Utc;
use base64::Engine;
use once_cell::sync::Lazy;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tempfile::tempdir;
use url::Url;

use crate::fs::{
    allowed_image_extension, backup_archive_name, create_backup_zip_file, db_path,
    ensure_storage_dirs, managed_release_image_path, restore_data_from_backup_zip,
};
use crate::models::{
    DashboardSummary, Release, ReleaseInput, ReleaseTrackRow, Track, TrackInput, TrackListRow,
};

const MIGRATION_001: &str = include_str!("../../migrations/001_initial_schema.sql");

#[derive(Deserialize)]
struct DriveUploadResponse {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct DriveFileListResponse {
    files: Vec<DriveFile>,
}

#[derive(Deserialize)]
struct DriveFile {
    id: String,
}

#[derive(Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
}

#[derive(Serialize, Deserialize)]
struct StoredGoogleTokens {
    access_token: String,
    refresh_token: Option<String>,
    expires_at_unix: i64,
}

struct PendingOAuthFlow {
    code_verifier: String,
    redirect_uri: String,
    code: Option<String>,
    error: Option<String>,
}

static OAUTH_FLOWS: Lazy<Mutex<HashMap<String, Arc<Mutex<PendingOAuthFlow>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn open_connection() -> Result<Connection, String> {
    ensure_storage_dirs()?;
    let path = db_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| e.to_string())?;
    Ok(conn)
}

pub fn initialize_database() -> Result<(), String> {
    let mut conn = open_connection()?;
    ensure_schema_migrations_table(&conn)?;
    apply_migration_if_needed(&mut conn, 1, "initial_schema", MIGRATION_001)?;
    Ok(())
}

fn ensure_schema_migrations_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(|e| e.to_string())
}

fn apply_migration_if_needed(
    conn: &mut Connection,
    version: i64,
    name: &str,
    sql: &str,
) -> Result<(), String> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT version FROM schema_migrations WHERE version = ?1",
            params![version],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if exists.is_some() {
        return Ok(());
    }

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute_batch(sql).map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT OR IGNORE INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
        params![version, name, now_iso()],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn trim_to_opt(value: &Option<String>) -> Option<String> {
    value.as_ref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn validate_track_input(input: &TrackInput) -> Result<(), String> {
    if input.internal_code.trim().is_empty() {
        return Err("Internal Code is required".into());
    }
    if input.title.trim().is_empty() {
        return Err("Title is required".into());
    }
    match input.status.as_str() {
        "Idea" | "Draft" | "In Progress" | "Final" => {}
        _ => return Err("Invalid Track status".into()),
    }
    if let Some(bpm) = input.bpm {
        if bpm <= 0 {
            return Err("BPM must be a positive integer".into());
        }
    }
    Ok(())
}

fn validate_release_input(input: &ReleaseInput) -> Result<(), String> {
    if input.internal_code.trim().is_empty() {
        return Err("Internal Code is required".into());
    }
    if input.title.trim().is_empty() {
        return Err("Title is required".into());
    }
    match input.r#type.as_str() {
        "Album" | "EP" | "Single" => {}
        _ => return Err("Invalid Release type".into()),
    }
    match input.status.as_str() {
        "Planned" | "In Progress" | "Released" => {}
        _ => return Err("Invalid Release status".into()),
    }
    Ok(())
}

fn map_track(row: &rusqlite::Row<'_>) -> rusqlite::Result<Track> {
    Ok(Track {
        id: row.get("id")?,
        internal_code: row.get("internal_code")?,
        title: row.get("title")?,
        status: row.get("status")?,
        description: row.get("description")?,
        lyrics: row.get("lyrics")?,
        notes: row.get("notes")?,
        bpm: row.get("bpm")?,
        key: row.get("musical_key")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn map_track_list_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TrackListRow> {
    let assigned_release_id: Option<i64> = row.get("assigned_release_id")?;
    Ok(TrackListRow {
        id: row.get("id")?,
        internal_code: row.get("internal_code")?,
        title: row.get("title")?,
        status: row.get("status")?,
        description: row.get("description")?,
        lyrics: row.get("lyrics")?,
        notes: row.get("notes")?,
        bpm: row.get("bpm")?,
        key: row.get("musical_key")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        assigned_release_id,
        assigned_release_title: row.get("assigned_release_title")?,
        availability: if assigned_release_id.is_some() {
            "Assigned".to_string()
        } else {
            "Available".to_string()
        },
    })
}

fn map_release(row: &rusqlite::Row<'_>) -> rusqlite::Result<Release> {
    Ok(Release {
        id: row.get("id")?,
        internal_code: row.get("internal_code")?,
        title: row.get("title")?,
        r#type: row.get("type")?,
        status: row.get("status")?,
        description: row.get("description")?,
        image_path: row.get("image_path")?,
        track_count: row.get("track_count")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn list_tracks() -> Result<Vec<TrackListRow>, String> {
    let conn = open_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.internal_code, t.title, t.status, t.description, t.lyrics, t.notes, t.bpm, t.musical_key,
                    t.created_at, t.updated_at, r.id AS assigned_release_id, r.title AS assigned_release_title
             FROM tracks t
             LEFT JOIN release_tracks rt ON rt.track_id = t.id
             LEFT JOIN releases r ON r.id = rt.release_id
             ORDER BY t.updated_at DESC, t.id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], map_track_list_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn get_track_by_id(id: i64) -> Result<Option<Track>, String> {
    let conn = open_connection()?;
    conn.query_row(
        "SELECT id, internal_code, title, status, description, lyrics, notes, bpm, musical_key, created_at, updated_at FROM tracks WHERE id = ?1",
        params![id],
        map_track,
    )
    .optional()
    .map_err(|e| e.to_string())
}

pub fn create_track(input: TrackInput) -> Result<Track, String> {
    validate_track_input(&input)?;
    let conn = open_connection()?;
    let now = now_iso();
    conn.execute(
        "INSERT INTO tracks (internal_code, title, status, description, lyrics, notes, bpm, musical_key, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            input.internal_code.trim(),
            input.title.trim(),
            input.status,
            trim_to_opt(&input.description),
            trim_to_opt(&input.lyrics),
            trim_to_opt(&input.notes),
            input.bpm,
            trim_to_opt(&input.key),
            now,
            now,
        ],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed: tracks.internal_code") {
            "Track Internal Code must be unique".to_string()
        } else {
            e.to_string()
        }
    })?;
    let id = conn.last_insert_rowid();
    get_track_by_id(id)?.ok_or_else(|| "Could not read created track".into())
}

pub fn update_track(id: i64, input: TrackInput) -> Result<Track, String> {
    validate_track_input(&input)?;
    let conn = open_connection()?;
    let updated = conn.execute(
        "UPDATE tracks
         SET internal_code = ?1,
             title = ?2,
             status = ?3,
             description = ?4,
             lyrics = ?5,
             notes = ?6,
             bpm = ?7,
             musical_key = ?8,
             updated_at = ?9
         WHERE id = ?10",
        params![
            input.internal_code.trim(),
            input.title.trim(),
            input.status,
            trim_to_opt(&input.description),
            trim_to_opt(&input.lyrics),
            trim_to_opt(&input.notes),
            input.bpm,
            trim_to_opt(&input.key),
            now_iso(),
            id,
        ],
    ).map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed: tracks.internal_code") {
            "Track Internal Code must be unique".to_string()
        } else {
            e.to_string()
        }
    })?;

    if updated == 0 {
        return Err("Track not found".into());
    }

    get_track_by_id(id)?.ok_or_else(|| "Could not read updated track".into())
}

pub fn delete_track(id: i64) -> Result<(), String> {
    let conn = open_connection()?;
    conn.execute("DELETE FROM tracks WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_available_tracks() -> Result<Vec<Track>, String> {
    let conn = open_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.internal_code, t.title, t.status, t.description, t.lyrics, t.notes, t.bpm, t.musical_key, t.created_at, t.updated_at
             FROM tracks t
             LEFT JOIN release_tracks rt ON rt.track_id = t.id
             WHERE rt.track_id IS NULL
             ORDER BY t.updated_at DESC, t.id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], map_track)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn search_tracks(query: String) -> Result<Vec<TrackListRow>, String> {
    let conn = open_connection()?;
    let q = format!("%{}%", query.trim());
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.internal_code, t.title, t.status, t.description, t.lyrics, t.notes, t.bpm, t.musical_key,
                    t.created_at, t.updated_at, r.id AS assigned_release_id, r.title AS assigned_release_title
             FROM tracks t
             LEFT JOIN release_tracks rt ON rt.track_id = t.id
             LEFT JOIN releases r ON r.id = rt.release_id
             WHERE t.internal_code LIKE ?1 OR t.title LIKE ?1
             ORDER BY t.updated_at DESC, t.id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![q], map_track_list_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn list_releases() -> Result<Vec<Release>, String> {
    let conn = open_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT r.id, r.internal_code, r.title, r.type, r.status, r.description, r.image_path,
                    COUNT(rt.id) AS track_count, r.created_at, r.updated_at
             FROM releases r
             LEFT JOIN release_tracks rt ON rt.release_id = r.id
             GROUP BY r.id, r.internal_code, r.title, r.type, r.status, r.description, r.image_path, r.created_at, r.updated_at
             ORDER BY r.updated_at DESC, r.id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], map_release)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn get_release_by_id(id: i64) -> Result<Option<Release>, String> {
    let conn = open_connection()?;
    conn.query_row(
        "SELECT r.id, r.internal_code, r.title, r.type, r.status, r.description, r.image_path,
                COUNT(rt.id) AS track_count, r.created_at, r.updated_at
         FROM releases r
         LEFT JOIN release_tracks rt ON rt.release_id = r.id
         WHERE r.id = ?1
         GROUP BY r.id, r.internal_code, r.title, r.type, r.status, r.description, r.image_path, r.created_at, r.updated_at",
        params![id],
        map_release,
    )
    .optional()
    .map_err(|e| e.to_string())
}

pub fn create_release(input: ReleaseInput) -> Result<Release, String> {
    validate_release_input(&input)?;
    let conn = open_connection()?;
    let now = now_iso();
    conn.execute(
        "INSERT INTO releases (internal_code, title, type, status, description, image_path, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            input.internal_code.trim(),
            input.title.trim(),
            input.r#type,
            input.status,
            trim_to_opt(&input.description),
            trim_to_opt(&input.image_path),
            now,
            now,
        ],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed: releases.internal_code") {
            "Release Internal Code must be unique".to_string()
        } else {
            e.to_string()
        }
    })?;
    let id = conn.last_insert_rowid();
    get_release_by_id(id)?.ok_or_else(|| "Could not read created release".into())
}

pub fn update_release(id: i64, input: ReleaseInput) -> Result<Release, String> {
    validate_release_input(&input)?;
    let conn = open_connection()?;
    let updated = conn.execute(
        "UPDATE releases
         SET internal_code = ?1,
             title = ?2,
             type = ?3,
             status = ?4,
             description = ?5,
             image_path = ?6,
             updated_at = ?7
         WHERE id = ?8",
        params![
            input.internal_code.trim(),
            input.title.trim(),
            input.r#type,
            input.status,
            trim_to_opt(&input.description),
            trim_to_opt(&input.image_path),
            now_iso(),
            id,
        ],
    ).map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed: releases.internal_code") {
            "Release Internal Code must be unique".to_string()
        } else {
            e.to_string()
        }
    })?;

    if updated == 0 {
        return Err("Release not found".into());
    }

    get_release_by_id(id)?.ok_or_else(|| "Could not read updated release".into())
}

pub fn delete_release(id: i64) -> Result<(), String> {
    let release = get_release_by_id(id)?;
    if let Some(release) = &release {
        if let Some(path) = &release.image_path {
            let _ = fs::remove_file(path);
        }
    }
    let conn = open_connection()?;
    conn.execute("DELETE FROM releases WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_tracks_for_release(release_id: i64) -> Result<Vec<ReleaseTrackRow>, String> {
    let conn = open_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT rt.track_order, t.id AS track_id, t.internal_code, t.title, t.status, t.bpm, t.musical_key
             FROM release_tracks rt
             JOIN tracks t ON t.id = rt.track_id
             WHERE rt.release_id = ?1
             ORDER BY rt.track_order ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![release_id], |row| {
            Ok(ReleaseTrackRow {
                track_order: row.get("track_order")?,
                track_id: row.get("track_id")?,
                internal_code: row.get("internal_code")?,
                title: row.get("title")?,
                status: row.get("status")?,
                bpm: row.get("bpm")?,
                key: row.get("musical_key")?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn assign_track_to_release(track_id: i64, release_id: i64) -> Result<(), String> {
    let conn = open_connection()?;
    // validate existence
    let track_exists: Option<i64> = conn
        .query_row("SELECT id FROM tracks WHERE id = ?1", params![track_id], |row| row.get(0))
        .optional()
        .map_err(|e| e.to_string())?;
    if track_exists.is_none() {
        return Err("Track not found".into());
    }

    let release_exists: Option<i64> = conn
        .query_row("SELECT id FROM releases WHERE id = ?1", params![release_id], |row| row.get(0))
        .optional()
        .map_err(|e| e.to_string())?;
    if release_exists.is_none() {
        return Err("Release not found".into());
    }

    let assigned: Option<i64> = conn
        .query_row(
            "SELECT release_id FROM release_tracks WHERE track_id = ?1",
            params![track_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if assigned.is_some() {
        return Err("Track is already assigned to a release".into());
    }

    let next_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(track_order), 0) + 1 FROM release_tracks WHERE release_id = ?1",
            params![release_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO release_tracks (release_id, track_id, track_order, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![release_id, track_id, next_order, now_iso()],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove_track_from_release(track_id: i64, release_id: i64) -> Result<(), String> {
    let conn = open_connection()?;
    let current_order: Option<i64> = conn
        .query_row(
            "SELECT track_order FROM release_tracks WHERE track_id = ?1 AND release_id = ?2",
            params![track_id, release_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let current_order = current_order.ok_or("Track is not assigned to this release")?;

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM release_tracks WHERE track_id = ?1 AND release_id = ?2",
        params![track_id, release_id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE release_tracks
         SET track_order = track_order - 1
         WHERE release_id = ?1 AND track_order > ?2",
        params![release_id, current_order],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn move_track_up_in_release(track_id: i64, release_id: i64) -> Result<(), String> {
    move_track_in_release(track_id, release_id, true)
}

pub fn move_track_down_in_release(track_id: i64, release_id: i64) -> Result<(), String> {
    move_track_in_release(track_id, release_id, false)
}

fn move_track_in_release(track_id: i64, release_id: i64, up: bool) -> Result<(), String> {
    let conn = open_connection()?;
    let current_order: Option<i64> = conn
        .query_row(
            "SELECT track_order FROM release_tracks WHERE track_id = ?1 AND release_id = ?2",
            params![track_id, release_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let current_order = current_order.ok_or("Track is not assigned to this release")?;
    let target_order = if up { current_order - 1 } else { current_order + 1 };
    if target_order <= 0 {
        return Ok(());
    }

    let other_track_id: Option<i64> = conn
        .query_row(
            "SELECT track_id FROM release_tracks WHERE release_id = ?1 AND track_order = ?2",
            params![release_id, target_order],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let other_track_id = match other_track_id {
        Some(v) => v,
        None => return Ok(()),
    };

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE release_tracks SET track_order = 0 WHERE release_id = ?1 AND track_id = ?2",
        params![release_id, track_id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE release_tracks SET track_order = ?3 WHERE release_id = ?1 AND track_id = ?2",
        params![release_id, other_track_id, current_order],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE release_tracks SET track_order = ?3 WHERE release_id = ?1 AND track_id = ?2",
        params![release_id, track_id, target_order],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn set_release_image(release_id: i64, source_path: String) -> Result<Release, String> {
    let source = Path::new(&source_path);
    if !source.exists() {
        return Err("Selected image file does not exist".into());
    }
    if !allowed_image_extension(source) {
        return Err("Unsupported image file type".into());
    }

    let release = get_release_by_id(release_id)?.ok_or("Release not found")?;
    let dest = managed_release_image_path(&release.internal_code, source)?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    if let Some(old) = &release.image_path {
        if old != dest.to_string_lossy().as_ref() {
            let _ = fs::remove_file(old);
        }
    }

    fs::copy(source, &dest).map_err(|e| e.to_string())?;

    let conn = open_connection()?;
    conn.execute(
        "UPDATE releases SET image_path = ?1, updated_at = ?2 WHERE id = ?3",
        params![dest.to_string_lossy().to_string(), now_iso(), release_id],
    )
    .map_err(|e| e.to_string())?;

    get_release_by_id(release_id)?.ok_or_else(|| "Could not read updated release".into())
}

pub fn remove_release_image(release_id: i64) -> Result<Release, String> {
    let release = get_release_by_id(release_id)?.ok_or("Release not found")?;
    if let Some(path) = &release.image_path {
        let _ = fs::remove_file(path);
    }
    let conn = open_connection()?;
    conn.execute(
        "UPDATE releases SET image_path = NULL, updated_at = ?1 WHERE id = ?2",
        params![now_iso(), release_id],
    )
    .map_err(|e| e.to_string())?;
    get_release_by_id(release_id)?.ok_or_else(|| "Could not read updated release".into())
}

pub fn start_google_drive_oauth() -> Result<String, String> {
    let client_id = google_oauth_client_id()?;

    let state = random_string(32);
    let code_verifier = random_string(96);
    let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(Sha256::digest(code_verifier.as_bytes()));

    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

    let flow = Arc::new(Mutex::new(PendingOAuthFlow {
        code_verifier,
        redirect_uri: redirect_uri.clone(),
        code: None,
        error: None,
    }));
    OAUTH_FLOWS
        .lock()
        .map_err(|_| "Could not lock OAuth flow store")?
        .insert(state.clone(), flow.clone());

    spawn_loopback_listener(listener, state.clone(), flow);

    let mut auth_url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
        .map_err(|e| e.to_string())?;
    auth_url
        .query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", "https://www.googleapis.com/auth/drive.file")
        .append_pair("state", &state)
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent");

    webbrowser::open(auth_url.as_str()).map_err(|e| e.to_string())?;
    Ok(state)
}

pub fn complete_google_drive_oauth(state: String) -> Result<bool, String> {
    let client_id = google_oauth_client_id()?;

    let flow_arc = {
        let flows = OAUTH_FLOWS
            .lock()
            .map_err(|_| "Could not lock OAuth flow store")?;
        flows
            .get(&state)
            .cloned()
            .ok_or("OAuth session not found. Start again.")?
    };

    let (code, verifier, redirect_uri, error) = {
        let flow = flow_arc.lock().map_err(|_| "Could not read OAuth state")?;
        (
            flow.code.clone(),
            flow.code_verifier.clone(),
            flow.redirect_uri.clone(),
            flow.error.clone(),
        )
    };

    if let Some(err) = error {
        remove_oauth_flow(&state)?;
        return Err(err);
    }
    let Some(code) = code else {
        return Ok(false);
    };

    let client = reqwest::blocking::Client::new();
    let token_response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code.as_str()),
            ("client_id", client_id.as_str()),
            ("code_verifier", verifier.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .map_err(|e| e.to_string())?;

    if !token_response.status().is_success() {
        let body = token_response.text().unwrap_or_else(|_| "".to_string());
        remove_oauth_flow(&state)?;
        return Err(format!("OAuth token exchange failed: {}", body));
    }

    let token_payload: GoogleTokenResponse = token_response.json().map_err(|e| e.to_string())?;
    persist_google_tokens(&StoredGoogleTokens {
        access_token: token_payload.access_token,
        refresh_token: token_payload.refresh_token,
        expires_at_unix: now_unix() + token_payload.expires_in,
    })?;
    remove_oauth_flow(&state)?;
    Ok(true)
}

pub fn backup_to_google_drive(folder_id: Option<String>) -> Result<String, String> {
    let token = get_valid_google_access_token()?;

    let tmp_dir = tempdir().map_err(|e| e.to_string())?;
    let backup_name = backup_archive_name();
    let archive_path = tmp_dir.path().join(&backup_name);
    create_backup_zip_file(&archive_path)?;
    let archive_bytes = fs::read(&archive_path).map_err(|e| e.to_string())?;

    let mut metadata = json!({
        "name": backup_name,
        "mimeType": "application/zip"
    });
    if let Some(folder) = folder_id
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        metadata["parents"] = json!([folder]);
    }

    let metadata_part =
        reqwest::blocking::multipart::Part::text(metadata.to_string()).mime_str("application/json")
            .map_err(|e| e.to_string())?;
    let file_part =
        reqwest::blocking::multipart::Part::bytes(archive_bytes).file_name(backup_name.clone());
    let form = reqwest::blocking::multipart::Form::new()
        .part("metadata", metadata_part)
        .part("file", file_part);

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart&fields=id,name")
        .bearer_auth(&token)
        .multipart(form)
        .send()
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let body = response.text().unwrap_or_else(|_| "".to_string());
        return Err(format!("Google Drive upload failed: {}", body));
    }

    let upload: DriveUploadResponse = response.json().map_err(|e| e.to_string())?;
    Ok(format!("Uploaded backup as {} ({})", upload.name, upload.id))
}

pub fn restore_latest_from_google_drive(folder_id: Option<String>) -> Result<(), String> {
    let token = get_valid_google_access_token()?;

    let mut query_parts = vec![
        "name contains 'tracklog-backup-'".to_string(),
        "trashed = false".to_string(),
    ];
    if let Some(folder) = folder_id
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        query_parts.push(format!("'{}' in parents", folder.replace('\'', "\\'")));
    }
    let query = query_parts.join(" and ");

    let client = reqwest::blocking::Client::new();
    let list_response = client
        .get("https://www.googleapis.com/drive/v3/files")
        .bearer_auth(&token)
        .query(&[
            ("q", query.as_str()),
            ("orderBy", "createdTime desc"),
            ("pageSize", "1"),
            ("fields", "files(id)"),
        ])
        .send()
        .map_err(|e| e.to_string())?;

    if !list_response.status().is_success() {
        let body = list_response.text().unwrap_or_else(|_| "".to_string());
        return Err(format!("Google Drive list failed: {}", body));
    }

    let list: DriveFileListResponse = list_response.json().map_err(|e| e.to_string())?;
    let newest = list
        .files
        .first()
        .ok_or("No TrackLog backup files found in Google Drive")?;

    let download_url = format!(
        "https://www.googleapis.com/drive/v3/files/{}?alt=media",
        newest.id
    );
    let download_response = client
        .get(download_url)
        .bearer_auth(&token)
        .send()
        .map_err(|e| e.to_string())?;

    if !download_response.status().is_success() {
        let body = download_response.text().unwrap_or_else(|_| "".to_string());
        return Err(format!("Google Drive download failed: {}", body));
    }

    let tmp_dir = tempdir().map_err(|e| e.to_string())?;
    let archive_path = tmp_dir.path().join("tracklog-restore.zip");
    let bytes = download_response.bytes().map_err(|e| e.to_string())?;
    fs::write(&archive_path, &bytes).map_err(|e| e.to_string())?;

    restore_data_from_backup_zip(Path::new(&archive_path))?;
    initialize_database()?;
    Ok(())
}

fn random_string(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn spawn_loopback_listener(listener: TcpListener, expected_state: String, flow: Arc<Mutex<PendingOAuthFlow>>) {
    thread::spawn(move || {
        let _ = listener.set_nonblocking(true);
        let start = SystemTime::now();
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0_u8; 4096];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let req = String::from_utf8_lossy(&buf[..n]).to_string();
                    let path = req
                        .lines()
                        .next()
                        .and_then(|line| line.split_whitespace().nth(1))
                        .unwrap_or("/");
                    let url = format!("http://localhost{}", path);

                    let (mut code, mut error, mut state) = (None, None, None);
                    if let Ok(parsed) = Url::parse(&url) {
                        for (k, v) in parsed.query_pairs() {
                            match k.as_ref() {
                                "code" => code = Some(v.to_string()),
                                "error" => error = Some(v.to_string()),
                                "state" => state = Some(v.to_string()),
                                _ => {}
                            }
                        }
                    }

                    if let Ok(mut data) = flow.lock() {
                        if state.as_deref() != Some(expected_state.as_str()) {
                            data.error = Some("OAuth state mismatch".to_string());
                        } else if let Some(err) = error {
                            data.error = Some(format!("OAuth authorization failed: {}", err));
                        } else {
                            data.code = code;
                        }
                    }

                    let body = "TrackLog authorization complete. You can close this tab.";
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes());
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if start.elapsed().unwrap_or(Duration::from_secs(0)) > Duration::from_secs(300) {
                        if let Ok(mut data) = flow.lock() {
                            data.error = Some("OAuth timed out. Start again.".to_string());
                        }
                        break;
                    }
                    thread::sleep(Duration::from_millis(150));
                }
                Err(_) => {
                    if let Ok(mut data) = flow.lock() {
                        data.error = Some("Failed waiting for OAuth callback".to_string());
                    }
                    break;
                }
            }
        }
    });
}

fn remove_oauth_flow(state: &str) -> Result<(), String> {
    OAUTH_FLOWS
        .lock()
        .map_err(|_| "Could not lock OAuth flow store".to_string())?
        .remove(state);
    Ok(())
}

fn google_tokens_path() -> Result<std::path::PathBuf, String> {
    let mut path = crate::fs::data_dir()?;
    path.push("google_drive_tokens.json");
    Ok(path)
}

fn persist_google_tokens(tokens: &StoredGoogleTokens) -> Result<(), String> {
    let path = google_tokens_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let payload = serde_json::to_vec_pretty(tokens).map_err(|e| e.to_string())?;
    fs::write(path, payload).map_err(|e| e.to_string())
}

fn load_google_tokens() -> Result<StoredGoogleTokens, String> {
    let path = google_tokens_path()?;
    let raw = fs::read(path).map_err(|_| "Google Drive is not connected yet".to_string())?;
    serde_json::from_slice(&raw).map_err(|e| e.to_string())
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn get_valid_google_access_token() -> Result<String, String> {
    let client_id = google_oauth_client_id()?;

    let mut stored = load_google_tokens()?;
    if stored.expires_at_unix > now_unix() + 60 {
        return Ok(stored.access_token);
    }

    let refresh_token = stored
        .refresh_token
        .clone()
        .ok_or("Access token expired and no refresh token is available. Reconnect Google Drive.")?;
    let refreshed = refresh_google_access_token(client_id.as_str(), &refresh_token)?;
    stored.access_token = refreshed.access_token;
    stored.expires_at_unix = now_unix() + refreshed.expires_in;
    if refreshed.refresh_token.is_some() {
        stored.refresh_token = refreshed.refresh_token;
    }
    persist_google_tokens(&stored)?;
    Ok(stored.access_token)
}

fn refresh_google_access_token(client_id: &str, refresh_token: &str) -> Result<GoogleTokenResponse, String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let body = response.text().unwrap_or_else(|_| "".to_string());
        return Err(format!("Failed to refresh Google access token: {}", body));
    }
    response.json().map_err(|e| e.to_string())
}

fn google_oauth_client_id() -> Result<String, String> {
    if let Ok(value) = std::env::var("TRACKLOG_GOOGLE_OAUTH_CLIENT_ID") {
        let trimmed = value.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    if let Some(value) = option_env!("TRACKLOG_GOOGLE_OAUTH_CLIENT_ID") {
        let trimmed = value.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    Err("Google OAuth client ID is not configured. Set TRACKLOG_GOOGLE_OAUTH_CLIENT_ID for the app.".into())
}

pub fn get_dashboard_summary() -> Result<DashboardSummary, String> {
    let conn = open_connection()?;

    let total_tracks: i64 = conn
        .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let available_tracks: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tracks t LEFT JOIN release_tracks rt ON rt.track_id = t.id WHERE rt.track_id IS NULL",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let total_releases: i64 = conn
        .query_row("SELECT COUNT(*) FROM releases", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

let recent_tracks = {
    let mut stmt = conn
        .prepare(
            "SELECT id, internal_code, title, status, description, lyrics, notes, bpm, musical_key, created_at, updated_at
             FROM tracks ORDER BY updated_at DESC, id DESC LIMIT 5",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], map_track)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    rows
};

let recent_releases = {
    let mut stmt = conn
        .prepare(
            "SELECT r.id, r.internal_code, r.title, r.type, r.status, r.description, r.image_path,
                    COUNT(rt.id) AS track_count, r.created_at, r.updated_at
             FROM releases r
             LEFT JOIN release_tracks rt ON rt.release_id = r.id
             GROUP BY r.id, r.internal_code, r.title, r.type, r.status, r.description, r.image_path, r.created_at, r.updated_at
             ORDER BY r.updated_at DESC, r.id DESC
             LIMIT 5",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], map_release)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    rows
};

    Ok(DashboardSummary {
        total_tracks,
        available_tracks,
        total_releases,
        recent_tracks,
        recent_releases,
    })
}
