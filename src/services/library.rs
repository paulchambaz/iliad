use anyhow::{bail, Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_yaml::Value;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::fs;
use std::path::Path;
use std::time::Duration;
use std::{fs::File, path::PathBuf};
use symphonia::core::{
    formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
};
use tar::Builder;

use crate::{models::audiobook::Audiobook, state::AppState};

pub async fn scan_library(state: &AppState, force: bool) -> Result<()> {
    let mut audiobooks: Vec<Audiobook> = Vec::new();

    if let Ok(entries) = fs::read_dir(&state.library_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                match scan_audiobook(&path, force) {
                    Ok(audiobook) => audiobooks.push(audiobook),
                    Err(e) => eprintln!("Error scanning audiobook in {:?}: {}", path, e),
                }
            }
        }
    }

    let db = &state.db;
    let existing_hashes: Vec<String> = sqlx::query_scalar("SELECT hash FROM audiobooks")
        .fetch_all(db)
        .await?;

    for audiobook in &audiobooks {
        if existing_hashes.contains(&audiobook.hash) {
            update_audiobook(db, audiobook).await?;
        } else {
            create_audiobook(db, audiobook).await?;
        }
    }

    delete_missing_audiobooks(db, &existing_hashes, &audiobooks).await?;

    Ok(())
}

pub async fn cleanup(state: &AppState) -> Result<()> {
    Ok(())
}

struct AudiobookChapter {
    pub title: String,
    pub path: String,
}

fn scan_audiobook(dir: &PathBuf, force: bool) -> Result<Audiobook> {
    let info_path = get_info_path(dir)?;

    let data = fs::read_to_string(&info_path)
        .with_context(|| format!("Failed to read file: {:?}", info_path))?;

    let yaml: Value = serde_yaml::from_str(&data)
        .with_context(|| format!("Failed to parse YAML from file: {:?}", info_path))?;

    let title = yaml["title"]
        .as_str()
        .context("Missing or invalid 'title'")?;

    let author = yaml["author"]
        .as_str()
        .context("Missing or invalid 'author'")?;

    let date = yaml["date"].as_i64().context("Missing or invalid 'date'")? as i32;

    let description = yaml["description"]
        .as_str()
        .context("Missing or invalid 'description'")?;

    let genres = yaml["genres"]
        .as_sequence()
        .context("Missing or invalid 'genres'")?
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(",");

    let chapters = yaml["chapters"]
        .as_sequence()
        .context("Missing or invalid 'chapters'")?;

    let mut audiobook_chapters: Vec<AudiobookChapter> = Vec::new();
    let mut total_duration = Duration::new(0, 0);
    let mut total_size = fs::metadata(&info_path)?.len();

    for chapter in chapters {
        let chapter_title = chapter["title"]
            .as_str()
            .context("Chapter missing 'title'")?;

        let chapter_path = chapter["path"].as_str().context("Chapter missing 'path'")?;

        let full_chapter_path = dir.join(chapter_path);

        if !full_chapter_path.exists() {
            bail!("Chapter file does not exist: {:?}", full_chapter_path);
        }

        let chapter_duration = compute_audio_duration(&full_chapter_path).with_context(|| {
            format!("Failed to compute duration for chapter: {}", chapter_title)
        })?;
        total_duration += chapter_duration;

        total_size += fs::metadata(&full_chapter_path)?.len();

        let chapter = AudiobookChapter {
            title: chapter_title.to_string(),
            path: chapter_path.to_string(),
        };

        audiobook_chapters.push(chapter);
    }

    let path = dir.to_string_lossy().into_owned();
    let hash = compute_hash(author, title, date);

    let audiobook = Audiobook {
        hash,
        title: title.to_string(),
        author: author.to_string(),
        date,
        description: description.to_string(),
        genres,
        duration: total_duration.as_secs(),
        size: total_size,
        path,
    };

    create_archive(&audiobook, audiobook_chapters, &info_path, dir, force)?;

    Ok(audiobook)
}

fn create_archive(
    audiobook: &Audiobook,
    chapters: Vec<AudiobookChapter>,
    info_path: &PathBuf,
    dir: &PathBuf,
    force: bool,
) -> Result<()> {
    let archive_path = dir.join(format!("{}.tar.gz", audiobook.hash));

    if !force && archive_path.exists() {
        return Ok(());
    }

    println!(
        "Creating archive for {} by {}",
        audiobook.title, audiobook.author
    );

    let archive_file = File::create(&archive_path)?;
    let enc = GzEncoder::new(archive_file, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_path_with_name(&info_path, "info.yml")?;

    for chapter in chapters {
        println!(
            "Adding chapter : {} of {} by {}",
            chapter.title, audiobook.title, audiobook.author
        );
        let chapter_path = dir.join(&chapter.path);
        tar.append_path_with_name(&chapter_path, chapter.path)?;
    }

    tar.finish()?;

    println!(
        "Finished scanning {} by {}",
        audiobook.title, audiobook.author
    );

    Ok(())
}

fn get_info_path(dir: &PathBuf) -> Result<PathBuf> {
    let info_path = ["info.yaml", "info.yml"]
        .iter()
        .map(|filename| dir.join(filename))
        .find(|path| path.exists())
        .context("Could not find info.yaml or info.yml")?;

    Ok(info_path)
}

fn compute_audio_duration(path: &Path) -> Result<Duration> {
    let src = std::fs::File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let hint = Hint::new();
    let format_opts: FormatOptions = Default::default();
    let metadata_opts: MetadataOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .context("Failed to probe audio file")?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .context("No supported audio tracks found")?;

    if let Some(duration) = track.codec_params.time_base.and_then(|tb| {
        track
            .codec_params
            .n_frames
            .map(|frames| tb.calc_time(frames))
    }) {
        return Ok(Duration::from_secs(duration.seconds) + Duration::from_secs_f64(duration.frac));
    }

    let mut total_duration = 0u64;
    let time_base = track
        .codec_params
        .time_base
        .context("Failed to get time base")?;

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
    result[..8].iter().map(|b| format!("{:02x}", b)).collect()
}

async fn update_audiobook(db: &SqlitePool, audiobook: &Audiobook) -> Result<()> {
    sqlx::query(
        "UPDATE audiobooks SET title = ?, author = ?, date = ?, description = ?, genres = ?, duration = ?, size = ?, path = ? WHERE hash = ?"
    )
    .bind(&audiobook.title)
    .bind(&audiobook.author)
    .bind(audiobook.date)
    .bind(&audiobook.description)
    .bind(&audiobook.genres)
    .bind(audiobook.duration as i64)
    .bind(audiobook.size as i64)
    .bind(&audiobook.path)
    .bind(&audiobook.hash)
    .execute(db)
    .await?;
    Ok(())
}

async fn create_audiobook(db: &SqlitePool, audiobook: &Audiobook) -> Result<()> {
    sqlx::query(
        "INSERT INTO audiobooks (hash, title, author, date, description, genres, duration, size, path) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&audiobook.hash)
    .bind(&audiobook.title)
    .bind(&audiobook.author)
    .bind(audiobook.date)
    .bind(&audiobook.description)
    .bind(&audiobook.genres)
    .bind(audiobook.duration as i64)
    .bind(audiobook.size as i64)
    .bind(&audiobook.path)
    .execute(db)
    .await?;
    Ok(())
}

async fn delete_missing_audiobooks(
    db: &SqlitePool,
    existing_hashes: &[String],
    audiobooks: &[Audiobook],
) -> Result<()> {
    let scanned_hashes: Vec<String> = audiobooks.iter().map(|a| a.hash.clone()).collect();
    for hash in existing_hashes {
        if !scanned_hashes.contains(hash) {
            sqlx::query("DELETE FROM audiobooks WHERE hash = ?")
                .bind(hash)
                .execute(db)
                .await?;
        }
    }
    Ok(())
}
