use dotenv::dotenv;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::warn;
use ring::digest::{Context, SHA256};
use std::env;
use std::process::Command;
use std::fs;
use std::path::{PathBuf, Path};
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
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(audiobook) = scan_audiobook_directory(&path) {
                    audiobooks.push(audiobook);
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

    match fs::read_to_string(info_path) {
        Ok(data) => match serde_yaml::from_str::<serde_yaml::Value>(&data) {
            Ok(metadata) => {
                let title = extract_string_property(&metadata, "title", dir)?;
                let author = extract_string_property(&metadata, "author", dir)?;
                let date = extract_string_property(&metadata, "date", dir)?;
                let description = extract_string_property(&metadata, "description", dir)?;
                let genres = extract_string_property(&metadata, "genres", dir)?;
                let duration = get_duration(&metadata, dir);
                let size = get_directory_size(dir);
                let path_str = dir.to_string_lossy().into_owned();
                let hash = compute_hash(&title, &author);

                Some(AudiobookScan {
                    hash,
                    title,
                    author,
                    date,
                    description,
                    genres,
                    duration,
                    size,
                    path: path_str,
                })
            }
            Err(e) => {
                warn!(
                    "Failed to parse info into YAML at directory: {}. Error: {}",
                    dir.display(),
                    e
                );
                None
            }
        },
        Err(e) => {
            warn!(
                "Failed to read into file at directory: {}. Error: {}",
                dir.display(),
                e
            );
            None
        }
    }
}

fn extract_string_property(
    metadata: &serde_yaml::Value,
    property: &str,
    dir: &Path,
) -> Option<String> {
    match metadata.get(property) {
        Some(serde_yaml::Value::String(s)) => Some(s.to_string()),
        _ => {
            warn!(
                "'{}' not found or not a string in YAML at directory: {}",
                property,
                dir.display()
            );
            None
        }
    }
}

pub fn get_directory_size(dir: &PathBuf) -> i32 {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|e| fs::metadata(e.path()).ok().map(|m| m.len() as i32))
        .sum()
}

pub fn get_duration(metadata: &serde_yaml::Value, dir: &Path) -> i32 {
    let chapters = metadata
        .get("chapters")
        .and_then(|c| c.as_sequence())
        .cloned()
        .unwrap_or_else(Vec::new);

    chapters
        .iter()
        .filter_map(|chapter| chapter.get("file").and_then(|f| f.as_str()))
        .map(|file_name| {
            let file_path = dir.join(file_name);
            get_audio_duration(&file_path)
        })
        .sum()
}

pub fn get_audio_duration(file_path: &Path) -> i32 {
    // TODO: as can be expected this is extremely slow
    // it needs to be rewritten to allow for a faster output
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(file_path.as_os_str())
        .output();

    match output {
        Ok(output_data) if output_data.status.success() => {
            let output_str = String::from_utf8_lossy(&output_data.stdout);
            let duration_str = output_str.trim();
            match duration_str.parse::<f64>() {
                Ok(duration) => duration.floor() as i32,
                _ => 0,
            }
        },
        _ => 0,
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
        if tar_builder.append_path_with_name(dir.join(&file), &file).is_err() {
            return Err(IliadError::ArchiveFailed);
        }
    }

    let encoder = match tar_builder.into_inner() {
        Ok(encoder) => encoder,
        Err(_) => return Err(IliadError::ArchiveFailed),
    };

    match encoder.finish() {
        Ok(compressed_data) => Ok(compressed_data),
        Err(_) => Err(IliadError::ArchiveFailed),
    }
}

#[cfg(test)]
mod tests {
    use super::get_audio_duration;
    use std::path::Path;

    #[test]
    fn test_get_audio_duration() {
        let duration = get_audio_duration(Path::new("./library/the-ancient-city-fustel-de-coulanges/introduction.opus"));

        println!("Duration of the audio file: {}", duration);
    }
}
