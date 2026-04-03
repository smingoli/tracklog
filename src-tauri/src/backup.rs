use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::fs::{app_root, data_dir};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub backup_location: Option<String>,
}

fn settings_path() -> Result<PathBuf, String> {
    let mut path = app_root()?;
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    path.push("settings.json");
    Ok(path)
}

pub fn get_backup_location() -> Result<Option<String>, String> {
    Ok(load_settings()?.backup_location)
}

pub fn set_backup_location(path: String) -> Result<Option<String>, String> {
    let target = PathBuf::from(path.trim());
    if !target.exists() || !target.is_dir() {
        return Err("Backup location must be an existing folder".into());
    }
    let mut settings = load_settings()?;
    settings.backup_location = Some(target.to_string_lossy().to_string());
    save_settings(&settings)?;
    Ok(settings.backup_location)
}

pub fn create_backup(destination_dir: String) -> Result<String, String> {
    let destination = PathBuf::from(destination_dir.trim());
    if !destination.exists() || !destination.is_dir() {
        return Err("Backup destination must be an existing folder".into());
    }

    let source_data_dir = data_dir()?;
    if !source_data_dir.exists() {
        return Err("Data directory does not exist yet".into());
    }

    let file_name = format!("tracklog-backup-{}.zip", Utc::now().format("%Y%m%d-%H%M%S"));
    let backup_path = destination.join(file_name);
    let backup_file = File::create(&backup_path).map_err(|e| e.to_string())?;
    let mut zip_writer = ZipWriter::new(backup_file);

    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(&source_data_dir) {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let rel = path
            .strip_prefix(&source_data_dir)
            .map_err(|e| e.to_string())?;
        if rel.as_os_str().is_empty() {
            continue;
        }

        let zip_path = format!("data/{}", rel.to_string_lossy().replace('\\', "/"));

        if entry.file_type().is_dir() {
            zip_writer
                .add_directory(format!("{zip_path}/"), options)
                .map_err(|e| e.to_string())?;
            continue;
        }

        zip_writer
            .start_file(zip_path, options)
            .map_err(|e| e.to_string())?;
        let mut f = File::open(path).map_err(|e| e.to_string())?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
        zip_writer.write_all(&buffer).map_err(|e| e.to_string())?;
    }

    zip_writer.finish().map_err(|e| e.to_string())?;
    Ok(backup_path.to_string_lossy().to_string())
}

pub fn restore_backup(backup_zip_path: String) -> Result<(), String> {
    let zip_path = PathBuf::from(backup_zip_path.trim());
    if !zip_path.exists() || !zip_path.is_file() {
        return Err("Backup ZIP file was not found".into());
    }

    let extraction_root = std::env::temp_dir().join(format!(
        "tracklog-restore-{}",
        Utc::now().format("%Y%m%d%H%M%S%3f")
    ));
    fs::create_dir_all(&extraction_root).map_err(|e| e.to_string())?;

    extract_zip(&zip_path, &extraction_root)?;

    let extracted_data = extraction_root.join("data");
    if !extracted_data.join("catalog.db").exists() {
        let _ = fs::remove_dir_all(&extraction_root);
        return Err("Invalid backup ZIP: catalog.db was not found in data/".into());
    }

    let current_data = data_dir()?;
    if current_data.exists() {
        fs::remove_dir_all(&current_data).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&current_data).map_err(|e| e.to_string())?;
    copy_dir_recursive(&extracted_data, &current_data)?;

    let _ = fs::remove_dir_all(&extraction_root);
    Ok(())
}

fn extract_zip(zip_path: &Path, target_root: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| "ZIP contained an invalid path".to_string())?;
        let out_path = target_root.join(enclosed);

        if entry.name().ends_with('/') {
            fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let mut out_file = File::create(&out_path).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out_file).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
    for entry in WalkDir::new(source) {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let rel = path.strip_prefix(source).map_err(|e| e.to_string())?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let target_path = target.join(rel);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(path, &target_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn load_settings() -> Result<AppSettings, String> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let serialized = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(settings_path()?, serialized).map_err(|e| e.to_string())
}
