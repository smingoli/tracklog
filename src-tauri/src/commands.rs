use tauri::command;

use crate::backup;
use crate::db;
use crate::models::{
    DashboardSummary, Release, ReleaseInput, ReleaseTrackRow, Track, TrackInput, TrackListRow,
};

#[command]
pub fn initialize_app() -> Result<(), String> {
    db::initialize_database()
}

#[command]
pub fn get_dashboard_summary() -> Result<DashboardSummary, String> {
    db::get_dashboard_summary()
}

#[command]
pub fn list_tracks() -> Result<Vec<TrackListRow>, String> {
    db::list_tracks()
}

#[command]
pub fn get_track_by_id(id: i64) -> Result<Option<Track>, String> {
    db::get_track_by_id(id)
}

#[command]
pub fn create_track(input: TrackInput) -> Result<Track, String> {
    db::create_track(input)
}

#[command]
pub fn update_track(id: i64, input: TrackInput) -> Result<Track, String> {
    db::update_track(id, input)
}

#[command]
pub fn delete_track(id: i64) -> Result<(), String> {
    db::delete_track(id)
}

#[command]
pub fn list_available_tracks() -> Result<Vec<Track>, String> {
    db::list_available_tracks()
}

#[command]
pub fn search_tracks(query: String) -> Result<Vec<TrackListRow>, String> {
    db::search_tracks(query)
}

#[command]
pub fn list_releases() -> Result<Vec<Release>, String> {
    db::list_releases()
}

#[command]
pub fn get_release_by_id(id: i64) -> Result<Option<Release>, String> {
    db::get_release_by_id(id)
}

#[command]
pub fn create_release(input: ReleaseInput) -> Result<Release, String> {
    db::create_release(input)
}

#[command]
pub fn update_release(id: i64, input: ReleaseInput) -> Result<Release, String> {
    db::update_release(id, input)
}

#[command]
pub fn delete_release(id: i64) -> Result<(), String> {
    db::delete_release(id)
}

#[command]
pub fn list_tracks_for_release(release_id: i64) -> Result<Vec<ReleaseTrackRow>, String> {
    db::list_tracks_for_release(release_id)
}

#[command]
pub fn assign_track_to_release(track_id: i64, release_id: i64) -> Result<(), String> {
    db::assign_track_to_release(track_id, release_id)
}

#[command]
pub fn remove_track_from_release(track_id: i64, release_id: i64) -> Result<(), String> {
    db::remove_track_from_release(track_id, release_id)
}

#[command]
pub fn move_track_up_in_release(track_id: i64, release_id: i64) -> Result<(), String> {
    db::move_track_up_in_release(track_id, release_id)
}

#[command]
pub fn move_track_down_in_release(track_id: i64, release_id: i64) -> Result<(), String> {
    db::move_track_down_in_release(track_id, release_id)
}

#[command]
pub fn set_release_image(release_id: i64, source_path: String) -> Result<Release, String> {
    db::set_release_image(release_id, source_path)
}

#[command]
pub fn remove_release_image(release_id: i64) -> Result<Release, String> {
    db::remove_release_image(release_id)
}

#[command]
pub fn write_text_file(path: String, contents: String) -> Result<(), String> {
    crate::fs::write_text_file(&path, &contents)
}

#[command]
pub fn get_backup_location() -> Result<Option<String>, String> {
    backup::get_backup_location()
}

#[command]
pub fn set_backup_location(path: String) -> Result<Option<String>, String> {
    backup::set_backup_location(path)
}

#[command]
pub fn get_backup_on_exit() -> Result<bool, String> {
    backup::get_backup_on_exit()
}

#[command]
pub fn set_backup_on_exit(enabled: bool) -> Result<bool, String> {
    backup::set_backup_on_exit(enabled)
}

#[command]
pub fn create_backup(destination_dir: String) -> Result<String, String> {
    backup::create_backup(destination_dir)
}

#[command]
pub fn restore_backup(backup_zip_path: String) -> Result<(), String> {
    backup::restore_backup(backup_zip_path)
}
