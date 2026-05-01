use crate::{
    error::AppError,
    models::audiobook::Audiobook,
    repo::{audiobook as audiobook_repo, position as position_repo},
    state::AppState,
};
use chrono::Utc;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_yml::Value;
use sha2::{Digest, Sha256};
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;
use std::time::Duration;
use std::{fs::File, path::PathBuf};
use symphonia::core::{
    formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
};
use tar::Builder;

pub async fn scan_library(state: &AppState) -> Result<(), AppError> {
    let dirs: Vec<PathBuf> = fs::read_dir(&state.library_path)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect()
        })
        .unwrap_or_default();

    let total_chapters: usize = dirs.iter().map(|d| count_chapters(d)).sum();

    let existing_map = audiobook_repo::find_hashes_with_checksums(&state.db).await?;

    let mut scanned: Vec<(Audiobook, String)> = Vec::new();
    let mut chapter_offset = 0usize;
    for dir in &dirs {
        match scan_audiobook(dir, chapter_offset, total_chapters) {
            Ok((audiobook, n, checksum)) => {
                chapter_offset += n;
                scanned.push((audiobook, checksum));
            }
            Err(e) => tracing::error!("scan failed for {:?}: {}", dir, e),
        }
    }

    for (audiobook, source_checksum) in &scanned {
        if existing_map.contains_key(&audiobook.hash) {
            audiobook_repo::update(&state.db, audiobook).await?;
        } else {
            audiobook_repo::create(&state.db, audiobook).await?;
        }

        let archive_path = PathBuf::from(&audiobook.path).join("archive.tar.gz");
        let (db_checksum, db_archive_ready) = existing_map
            .get(&audiobook.hash)
            .map(|e| (e.0.as_deref(), e.1))
            .unwrap_or((None, false));
        let archive_exists = archive_path.exists();

        tracing::debug!(
            hash = %audiobook.hash,
            title = %audiobook.title,
            source_checksum = %source_checksum,
            db_checksum = ?db_checksum,
            db_archive_ready,
            archive_exists,
            "archive staleness check"
        );

        let archive_current = archive_exists
            && db_checksum == Some(source_checksum.as_str())
            && db_archive_ready;

        if archive_current {
            tracing::info!(hash = %audiobook.hash, "archive up-to-date, skipping");
            audiobook_repo::mark_ready(&state.db, &audiobook.hash).await?;
        } else {
            tracing::info!(
                hash = %audiobook.hash,
                reason = if !archive_exists { "no archive on disk" } else if db_checksum.is_none() { "no db checksum" } else { "checksum mismatch" },
                "archive stale, queuing"
            );
            audiobook_repo::mark_pending(&state.db, &audiobook.hash, source_checksum).await?;
            enqueue_archive(state, &audiobook.hash);
        }
    }

    let scanned_hashes: Vec<&str> = scanned.iter().map(|(a, _)| a.hash.as_str()).collect();
    for hash in existing_map.keys() {
        if !scanned_hashes.contains(&hash.as_str()) {
            audiobook_repo::delete(&state.db, hash).await?;
        }
    }

    Ok(())
}

pub async fn cleanup(state: &AppState) -> Result<(), AppError> {
    let cutoff_date = Utc::now().naive_utc() - chrono::Duration::days(3 * 365);
    position_repo::delete_old(&state.db, cutoff_date).await?;
    position_repo::delete_beyond_final(&state.db).await?;
    Ok(())
}

struct AudiobookChapter {
    pub title: String,
    pub path: String,
}

fn count_chapters(dir: &Path) -> usize {
    let info_path = match get_info_path(dir) {
        Ok(p) => p,
        Err(_) => return 0,
    };
    let data = match fs::read_to_string(&info_path) {
        Ok(d) => d,
        Err(_) => return 0,
    };
    let yaml: Value = match serde_yml::from_str(&data) {
        Ok(v) => v,
        Err(_) => return 0,
    };
    yaml["chapters"]
        .as_sequence()
        .map(|s| s.len())
        .unwrap_or(0)
}

fn scan_audiobook(
    dir: &Path,
    chapter_offset: usize,
    total_chapters: usize,
) -> Result<(Audiobook, usize, String), AppError> {
    let info_path = get_info_path(dir)?;

    let data = fs::read_to_string(&info_path)
        .map_err(|e| AppError::Internal(format!("failed to read {:?}: {e}", info_path)))?;

    let yaml: Value = serde_yml::from_str(&data)
        .map_err(|e| AppError::Internal(format!("failed to parse YAML from {:?}: {e}", info_path)))?;

    let title = yaml["title"]
        .as_str()
        .ok_or_else(|| AppError::Internal("missing or invalid 'title'".into()))?;

    let author = yaml["author"]
        .as_str()
        .ok_or_else(|| AppError::Internal("missing or invalid 'author'".into()))?;

    let date = yaml["date"]
        .as_i64()
        .ok_or_else(|| AppError::Internal("missing or invalid 'date'".into()))? as i32;

    let description = yaml["description"]
        .as_str()
        .ok_or_else(|| AppError::Internal("missing or invalid 'description'".into()))?;

    let genres = serde_json::to_string(
        &yaml["genres"]
            .as_sequence()
            .ok_or_else(|| AppError::Internal("missing or invalid 'genres'".into()))?
            .iter()
            .filter_map(|v: &serde_yml::Value| v.as_str())
            .collect::<Vec<_>>(),
    )
    .map_err(|e| AppError::Internal(format!("failed to serialize genres: {e}")))?;

    let chapters = yaml["chapters"]
        .as_sequence()
        .ok_or_else(|| AppError::Internal("missing or invalid 'chapters'".into()))?;

    let mut chapter_paths: Vec<PathBuf> = Vec::new();
    let mut chapter_durations: Vec<Duration> = Vec::new();
    let mut total_duration = Duration::new(0, 0);
    let mut total_size = fs::metadata(&info_path)?.len();
    let mut final_chapter_index = 0;
    let mut final_chapter_position = 0;

    for (i, chapter) in chapters.iter().enumerate() {
        let chapter_title = chapter["title"]
            .as_str()
            .ok_or_else(|| AppError::Internal("chapter missing 'title'".into()))?;

        let chapter_path = chapter["path"]
            .as_str()
            .ok_or_else(|| AppError::Internal("chapter missing 'path'".into()))?;

        let full_chapter_path = dir.join(chapter_path);

        if !full_chapter_path.exists() {
            return Err(AppError::Internal(format!(
                "chapter file does not exist: {:?}",
                full_chapter_path
            )));
        }

        tracing::info!(
            "[{}/{}] reading duration: {}",
            chapter_offset + i + 1,
            total_chapters,
            chapter_title
        );

        let chapter_duration = compute_audio_duration(&full_chapter_path).map_err(|e| {
            AppError::Internal(format!("failed to compute duration for '{}': {e}", chapter_title))
        })?;

        total_duration += chapter_duration;
        total_size += fs::metadata(&full_chapter_path)?.len();
        chapter_durations.push(chapter_duration);
        chapter_paths.push(full_chapter_path);
    }

    let five_minutes = Duration::from_secs(5 * 60);
    if total_duration > five_minutes {
        let mut remaining = five_minutes;
        for (index, dur) in chapter_durations.iter().enumerate().rev() {
            if *dur > remaining {
                final_chapter_index = index as i64;
                final_chapter_position = (*dur - remaining).as_millis() as i64;
                break;
            }
            remaining -= *dur;
        }
    }

    let source_checksum = compute_source_checksum(&info_path, &chapter_paths);
    let path = dir.to_string_lossy().into_owned();
    let hash = compute_hash(author, title, date);

    let audiobook = Audiobook {
        hash,
        title: title.to_string(),
        author: author.to_string(),
        date,
        description: description.to_string(),
        genres,
        duration: total_duration.as_secs() as i64,
        size: total_size as i64,
        path,
        final_chapter_index,
        final_chapter_position,
        archive_ready: false,
    };

    Ok((audiobook, chapter_durations.len(), source_checksum))
}

pub fn build_archive(dir: &Path) -> Result<(), AppError> {
    create_archive(dir)
}

fn create_archive(dir: &Path) -> Result<(), AppError> {
    let info_path = get_info_path(dir)?;

    let data = fs::read_to_string(&info_path)
        .map_err(|e| AppError::Internal(format!("failed to read {:?}: {e}", info_path)))?;

    let yaml: Value = serde_yml::from_str(&data)
        .map_err(|e| AppError::Internal(format!("failed to parse YAML: {e}")))?;

    let title = yaml["title"].as_str().unwrap_or("unknown");
    let author = yaml["author"].as_str().unwrap_or("unknown");
    let chapters = yaml["chapters"]
        .as_sequence()
        .ok_or_else(|| AppError::Internal("missing 'chapters'".into()))?;

    let chapter_list: Vec<AudiobookChapter> = chapters
        .iter()
        .filter_map(|c| {
            Some(AudiobookChapter {
                title: c["title"].as_str()?.to_string(),
                path: c["path"].as_str()?.to_string(),
            })
        })
        .collect();

    let n = chapter_list.len();
    let archive_path = dir.join("archive.tar.gz");
    tracing::info!("creating archive for {} by {}", title, author);
    let archive_file = File::create(&archive_path)?;
    let enc = GzEncoder::new(archive_file, Compression::fast());
    let mut tar = Builder::new(enc);
    tar.append_path_with_name(&info_path, "info.yml")?;

    for (i, chapter) in chapter_list.into_iter().enumerate() {
        tracing::info!("[{}/{}] archiving: {}", i + 1, n, chapter.title);
        let full_path = dir.join(&chapter.path);
        tar.append_path_with_name(&full_path, chapter.path)?;
    }

    tar.finish()?;
    Ok(())
}

fn compute_source_checksum(info_path: &Path, chapter_paths: &[PathBuf]) -> String {
    let mut hasher = Sha256::new();
    match fs::read(info_path) {
        Ok(content) => {
            tracing::debug!(path = ?info_path, bytes = content.len(), "checksum: hashing info file");
            hasher.update(&content);
        }
        Err(e) => tracing::warn!(path = ?info_path, error = %e, "checksum: failed to read info file"),
    }
    hasher.update(b"\n");
    for path in chapter_paths {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        tracing::debug!(name = %name, size = size, "checksum: hashing chapter");
        hasher.update(name.as_bytes());
        hasher.update(b":");
        hasher.update(size.to_le_bytes());
        hasher.update(b"\n");
    }
    let result = hasher.finalize();
    result[..8].iter().fold(String::with_capacity(16), |mut s, b| {
        write!(s, "{b:02x}").unwrap();
        s
    })
}

fn get_info_path(dir: &Path) -> Result<PathBuf, AppError> {
    ["info.yaml", "info.yml"]
        .iter()
        .map(|filename| dir.join(filename))
        .find(|path| path.exists())
        .ok_or_else(|| AppError::Internal("could not find info.yaml or info.yml".into()))
}

fn compute_audio_duration(path: &Path) -> Result<Duration, AppError> {
    let src = std::fs::File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| AppError::Internal(format!("failed to probe audio file: {e}")))?;
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| AppError::Internal("no supported audio tracks found".into()))?;

    if let Some(duration) = track.codec_params.time_base.and_then(|tb| {
        track
            .codec_params
            .n_frames
            .map(|frames| tb.calc_time(frames))
    }) {
        return Ok(Duration::from_secs(duration.seconds) + Duration::from_secs_f64(duration.frac));
    }

    let time_base = track
        .codec_params
        .time_base
        .ok_or_else(|| AppError::Internal("failed to get time base".into()))?;

    let mut total_duration = 0u64;
    while let Ok(packet) = format.next_packet() {
        total_duration += packet.dur();
    }

    let duration = time_base.calc_time(total_duration);
    Ok(Duration::from_secs(duration.seconds) + Duration::from_secs_f64(duration.frac))
}

fn compute_hash(author: &str, title: &str, date: i32) -> String {
    let mut hasher = Sha256::new();
    hasher.update(author);
    hasher.update(title);
    hasher.update(date.to_string());
    let result = hasher.finalize();
    result[..8].iter().fold(String::with_capacity(16), |mut s, b| {
        write!(s, "{b:02x}").unwrap();
        s
    })
}

fn enqueue_archive(state: &AppState, hash: &str) {
    let mut q = state.archive_queue.lock().unwrap();
    if q.in_progress.as_deref() != Some(hash) && !q.pending.contains(&hash.to_string()) {
        q.pending.push_back(hash.to_string());
        drop(q);
        state.archive_notify.notify_one();
    }
}
