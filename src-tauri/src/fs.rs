use dirs::data_local_dir;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

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
    match path
        .extension()
        .and_then(OsStr::to_str)
        .map(|s| s.to_lowercase())
    {
        Some(ext) if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp") => true,
        _ => false,
    }
}

pub fn managed_release_image_path(
    release_internal_code: &str,
    source: &Path,
) -> Result<PathBuf, String> {
    let ext = source
        .extension()
        .and_then(OsStr::to_str)
        .ok_or("Image file has no valid extension")?;
    let mut dst = releases_image_dir()?;
    dst.push(format!(
        "{}.{}",
        sanitize_filename(release_internal_code),
        ext.to_lowercase()
    ));
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

pub fn write_text_file(path: &str, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e| e.to_string())
}
