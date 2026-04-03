mod backup;
mod commands;
mod db;
mod fs;
mod models;

use commands::{
    assign_track_to_release, create_backup, create_release, create_track, delete_release,
    delete_track, get_backup_location, get_dashboard_summary, get_release_by_id, get_track_by_id,
    get_backup_on_exit, initialize_app, list_available_tracks, list_releases, list_tracks,
    list_tracks_for_release, move_track_down_in_release, move_track_up_in_release,
    remove_release_image, remove_track_from_release, restore_backup, search_tracks,
    set_backup_location, set_backup_on_exit, set_release_image, update_release, update_track,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            initialize_app,
            get_dashboard_summary,
            list_tracks,
            get_track_by_id,
            create_track,
            update_track,
            delete_track,
            list_available_tracks,
            search_tracks,
            list_releases,
            get_release_by_id,
            create_release,
            update_release,
            delete_release,
            list_tracks_for_release,
            assign_track_to_release,
            remove_track_from_release,
            move_track_up_in_release,
            move_track_down_in_release,
            set_release_image,
            remove_release_image,
            get_backup_location,
            get_backup_on_exit,
            set_backup_location,
            set_backup_on_exit,
            create_backup,
            restore_backup
        ])
        .build(tauri::generate_context!())
        .expect("error while building SongsCatalog")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                if let Err(err) = backup::run_backup_on_exit_if_enabled() {
                    eprintln!("backup on exit failed: {err}");
                }
            }
        });
}
