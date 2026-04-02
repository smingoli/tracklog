use dirs::data_local_dir;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn app_root() -> Result<PathBuf, String> {
    let mut root = data_local_dir().ok_or("Could not resolve Local AppData path")?;
    root.push("TrackLog");
    Ok(root)
}

pub fn data_dir() -> Result<PathBuf, String> {
    let mut p = app_root()?;
    p.push("data");
    Ok(p)
}

pub fn releases_image_dir() -> Result<PathBuf, String> {
    let mut p = data_dir()?;
    p.push("images");
    p.push("releases");
    Ok(p)
}

pub fn db_path() -> Result<PathBuf, String> {
    let mut p = data_dir()?;
    p.push("catalog.db");
    Ok(p)
}

pub fn ensure_storage_dirs() -> Result<(), String> {
    let data = data_dir()?;
    let images = releases_image_dir()?;
    fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    fs::create_dir_all(&images).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn allowed_image_extension(path: &Path) -> bool {
    match path.extension().and_then(OsStr::to_str).map(|s| s.to_lowercase()) {
        Some(ext) if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp") => true,
        _ => false,
    }
}

pub fn managed_release_image_path(release_internal_code: &str, source: &Path) -> Result<PathBuf, String> {
    let ext = source
        .extension()
        .and_then(OsStr::to_str)
        .ok_or("Image file has no valid extension")?;
    let mut dst = releases_image_dir()?;
    dst.push(format!("{}.{}", sanitize_filename(release_internal_code), ext.to_lowercase()));
    Ok(dst)
}

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn backup_data_to_directory(target_dir: &Path) -> Result<PathBuf, String> {
    if !target_dir.exists() || !target_dir.is_dir() {
        return Err("Selected backup destination is not a valid folder".into());
    }

    let source = data_dir()?;
    if !source.exists() {
        return Err("No local TrackLog data found to back up".into());
    }

    let backup_name = format!("tracklog-backup-{}", unix_timestamp());
    let backup_root = target_dir.join(backup_name);
    copy_directory_recursive(&source, &backup_root)?;
    Ok(backup_root)
}

pub fn restore_data_from_directory(backup_root: &Path) -> Result<(), String> {
    if !backup_root.exists() || !backup_root.is_dir() {
        return Err("Selected backup folder is not valid".into());
    }

    let backup_db = backup_root.join("catalog.db");
    if !backup_db.exists() {
        return Err("Backup folder does not contain catalog.db".into());
    }

    let destination = data_dir()?;
    if destination.exists() {
        fs::remove_dir_all(&destination).map_err(|e| e.to_string())?;
    }

    copy_directory_recursive(backup_root, &destination)?;
    Ok(())
}

fn copy_directory_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;

    let entries = fs::read_dir(source).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_type = entry.file_type().map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = destination.join(entry.file_name());

        if entry_type.is_dir() {
            copy_directory_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
