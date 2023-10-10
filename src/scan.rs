use dotenv::dotenv;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::warn;
use ring::digest::{Context, SHA256};
use serde_yaml;
use std::env;
use std::fs;
use std::path::PathBuf;
use tar::Builder;
use walkdir::WalkDir;

use crate::database;
use crate::errors::IliadError;
use crate::models::AudiobookScan;

/// Scans the audiobook directory as defined in the environment, and updates the database.
///
/// This function reads the directory path from the "ILIAD_LIBRARY_PATH" environment variable, then
/// iterates throught the directory entries, scanning each sub-directory for audiobook details.
/// After collecting the audiobooks, it updates the database with the new information.
pub fn scan_audiobooks_directory() -> Result<(), ()> {
    dotenv().ok();

    let library_url = match env::var("ILIAD_LIBRARY_PATH") {
        Ok(url) => url,
        Err(_) => return Err(()),
    };

    let mut audiobooks: Vec<AudiobookScan> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(library_url) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(audiobook) = scan_audiobook_directory(&path) {
                        audiobooks.push(audiobook);
                    }
                }
            }
        }
    }

    let conn = &mut match database::establish_connection() {
        Ok(conn) => conn,
        Err(_) => return Err(()),
    };

    match database::update_audiobooks(conn, audiobooks) {
        Ok(_) => {}
        Err(_) => return Err(()),
    };

    Ok(())
}

/// Scans a specific directory for an audiobook's metadata
///
/// The function looks for an 'info.yaml' or 'info.yml' file in the given directory.
/// If found, it extracts the title and author, generates a unique hash for the audiobook, and
/// returns the structured representation of the audiobook.
///
/// # Arguments
///
/// * `dir` - The dir to be scanned for an audiobook
///
/// # Returns
///
/// On success, the structured representation of the audiobook. On failure, when no info file is
/// found, when the file could not be read, or when the metadata was incomplete or malformed, None.
pub fn scan_audiobook_directory(dir: &PathBuf) -> Option<AudiobookScan> {
    let info_path_yaml = dir.join("info.yaml");
    let info_path_yml = dir.join("info.yml");

    let info_path = if info_path_yaml.exists() {
        &info_path_yaml
    } else if info_path_yml.exists() {
        &info_path_yml
    } else {
        return None;
    };

    let path_str = dir.to_string_lossy().into_owned();

    match fs::read_to_string(info_path) {
        Ok(data) => match serde_yaml::from_str::<serde_yaml::Value>(&data) {
            Ok(metadata) => {
                let title = match metadata.get("title") {
                    Some(serde_yaml::Value::String(s)) => s.to_string(),
                    _ => {
                        warn!(
                            "'title' not found or not a string in YAML at directory: {}",
                            dir.display()
                        );
                        return None;
                    }
                };

                let author = match metadata.get("author") {
                    Some(serde_yaml::Value::String(s)) => s.to_string(),
                    _ => {
                        warn!(
                            "'author' not found or not a string in YAML at directory: {}",
                            dir.display()
                        );
                        return None;
                    }
                };

                let hash = compute_hash(&title, &author);

                let audiobook = AudiobookScan {
                    hash,
                    title,
                    author,
                    path: path_str,
                };

                Some(audiobook)
            }
            Err(e) => {
                warn!(
                    "Failed to parse info YAML at directory: {}. Error: {}",
                    dir.display(),
                    e
                );
                None
            }
        },
        Err(e) => {
            warn!(
                "Failed to read info file at directory: {}. Error: {}",
                dir.display(),
                e
            );
            None
        }
    }
}

/// Generates a unique hash for an audiobook based on its title and author.
///
/// # Arguments
///
/// * `title`  - The author name, used to produce a unique identifier
/// * `author` - The title name, used to produce a unique identifier
///
/// # Returns
///
/// The unique hash for the audiobook
fn compute_hash(title: &str, author: &str) -> String {
    let mut context = Context::new(&SHA256);
    context.update(title.as_bytes());
    context.update(author.as_bytes());
    let digest = context.finish();
    hex::encode(digest.as_ref())[0..32].to_string()
}

/// Archive a directory into a compressed format.
///
/// Takes a directory and produce a Gzipped TAR archive as binary data containing all the files
/// within that directory.
///
/// # Parameters
///
/// * `dir` - The path of the directory to archive
///
/// # Returns
///
/// On success, the binary data of the Gzipped TAR archive. On failure, the relevant error code.
pub fn archive_directory(dir: &PathBuf) -> Result<Vec<u8>, IliadError> {
    let files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| entry.path().strip_prefix(dir).ok().map(|p| p.to_path_buf()))
        .collect();

    let compressed_data = Vec::new();
    let encoder = GzEncoder::new(compressed_data, Compression::default());
    let mut tar_builder = Builder::new(encoder);

    for file in files {
        if let Err(_) = tar_builder.append_path_with_name(dir.join(&file), &file) {
            return Err(IliadError::ArchiveFailed);
        }
    }

    let encoder = match tar_builder.into_inner() {
        Ok(encoder) => encoder,
        Err(_) => return Err(IliadError::ArchiveFailed),
    };

    match encoder.finish() {
        Ok(compressed_data) => Ok(compressed_data),
        Err(_) => return Err(IliadError::ArchiveFailed),
    }
}
