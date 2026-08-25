use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::fs::{data_dir, db_path};

pub fn export_for_companion(destination_path: String) -> Result<String, String> {
    let destination = PathBuf::from(destination_path.trim());
    validate_destination(&destination)?;

    let database = db_path()?;
    if !database.is_file() {
        return Err("catalog.db does not exist yet".into());
    }

    let export_file = File::create(&destination).map_err(|e| e.to_string())?;
    let mut zip_writer = ZipWriter::new(export_file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    add_file(&mut zip_writer, &database, "catalog.db", options)?;

    let images = data_dir()?.join("images");
    if images.is_dir() {
        add_directory_tree(&mut zip_writer, &images, "images", options)?;
    } else {
        zip_writer
            .add_directory("images/", options)
            .map_err(|e| e.to_string())?;
    }

    zip_writer.finish().map_err(|e| e.to_string())?;
    Ok(destination.to_string_lossy().to_string())
}

fn validate_destination(destination: &Path) -> Result<(), String> {
    if destination.as_os_str().is_empty() {
        return Err("Export destination is required".into());
    }
    if destination.extension().and_then(|ext| ext.to_str()) != Some("zip") {
        return Err("Export destination must use the .zip extension".into());
    }
    let parent = destination
        .parent()
        .ok_or_else(|| "Export destination must have a parent folder".to_string())?;
    if !parent.is_dir() {
        return Err("Export destination folder does not exist".into());
    }
    Ok(())
}

fn add_directory_tree(
    zip_writer: &mut ZipWriter<File>,
    source: &Path,
    archive_root: &str,
    options: FileOptions,
) -> Result<(), String> {
    zip_writer
        .add_directory(format!("{archive_root}/"), options)
        .map_err(|e| e.to_string())?;

    for entry in WalkDir::new(source) {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let relative = path.strip_prefix(source).map_err(|e| e.to_string())?;
        if relative.as_os_str().is_empty() {
            continue;
        }

        let archive_path = format!(
            "{archive_root}/{}",
            relative.to_string_lossy().replace('\\', "/")
        );
        if entry.file_type().is_dir() {
            zip_writer
                .add_directory(format!("{archive_path}/"), options)
                .map_err(|e| e.to_string())?;
        } else {
            add_file(zip_writer, path, &archive_path, options)?;
        }
    }
    Ok(())
}

fn add_file(
    zip_writer: &mut ZipWriter<File>,
    source: &Path,
    archive_path: &str,
    options: FileOptions,
) -> Result<(), String> {
    zip_writer
        .start_file(archive_path, options)
        .map_err(|e| e.to_string())?;
    let mut source_file = File::open(source).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    source_file
        .read_to_end(&mut buffer)
        .map_err(|e| e.to_string())?;
    zip_writer.write_all(&buffer).map_err(|e| e.to_string())
}
