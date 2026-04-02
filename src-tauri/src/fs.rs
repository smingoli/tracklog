use dirs::data_local_dir;
use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

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

pub fn create_backup_zip_file(target_file: &Path) -> Result<(), String> {
    let source = data_dir()?;
    if !source.exists() {
        return Err("No local TrackLog data found to back up".into());
    }

    if let Some(parent) = target_file.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file = fs::File::create(target_file).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(&source) {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let relative = path
            .strip_prefix(&source)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");

        if relative.is_empty() {
            continue;
        }

        if entry.file_type().is_dir() {
            zip.add_directory(relative, options).map_err(|e| e.to_string())?;
            continue;
        }

        zip.start_file(relative, options).map_err(|e| e.to_string())?;
        let mut src_file = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        src_file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        zip.write_all(&buf).map_err(|e| e.to_string())?;
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn restore_data_from_backup_zip(zip_path: &Path) -> Result<(), String> {
    if !zip_path.exists() {
        return Err("Backup archive was not found".into());
    }

    let file = fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let destination = data_dir()?;
    if destination.exists() {
        fs::remove_dir_all(&destination).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&destination).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut item = archive.by_index(i).map_err(|e| e.to_string())?;
        let enclosed = item.enclosed_name().ok_or("Invalid entry in backup zip")?;
        let out_path = destination.join(enclosed);
        if item.name().ends_with('/') {
            fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut out_file = fs::File::create(&out_path).map_err(|e| e.to_string())?;
        std::io::copy(&mut item, &mut out_file).map_err(|e| e.to_string())?;
    }

    if !destination.join("catalog.db").exists() {
        return Err("Backup archive did not contain catalog.db".into());
    }

    Ok(())
}

pub fn backup_archive_name() -> String {
    format!("tracklog-backup-{}.zip", unix_timestamp())
}

pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
