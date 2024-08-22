use anyhow::{Context, Result};
use slug::slugify;
use std::{fs, path::PathBuf};

use crate::{
    models::audiobook::Audiobook,
    outputs::audiobook::{AudiobookLong, AudiobookShort},
    state::AppState,
};

pub async fn list_audiobooks(state: &AppState) -> Result<Vec<AudiobookShort>> {
    let db = &state.db;

    let audiobooks = sqlx::query_as::<_, Audiobook>("SELECT * FROM audiobooks")
        .fetch_all(db)
        .await
        .context("Failed to fetch audiobooks")?;

    Ok(audiobooks
        .into_iter()
        .map(|book| AudiobookShort {
            hash: book.hash,
            title: book.title,
            author: book.author,
        })
        .collect())
}

pub async fn get_audiobook_by_hash(hash: String, state: &AppState) -> Result<AudiobookLong> {
    let db = &state.db;

    let book = sqlx::query_as::<_, Audiobook>("SELECT * FROM audiobooks WHERE hash = ?")
        .bind(&hash)
        .fetch_optional(db)
        .await
        .context("Failed to fetch audiobook")?
        .ok_or_else(|| anyhow::anyhow!("Audiobook not found"))?;

    Ok(AudiobookLong {
        hash: book.hash,
        title: book.title,
        author: book.author,
        date: book.date,
        description: book.description,
        genres: book.genres.split(',').map(String::from).collect(),
        duration: book.duration,
        size: book.size,
    })
}

pub async fn get_audiobook_archive(hash: String, state: &AppState) -> Result<(String, Vec<u8>)> {
    let db = &state.db;
    let audiobook = sqlx::query_as::<_, Audiobook>("SELECT * FROM audiobooks WHERE hash = ?")
        .bind(&hash)
        .fetch_one(db)
        .await
        .context("Failed to fetch audiobook from database")?;

    let audiobook_path = PathBuf::from(audiobook.path);
    let archive_path = audiobook_path.join("archive.tar.gz");

    let archive_content = fs::read(&archive_path).context("Failed to read archive file")?;

    let filename = format!(
        "{}-{}-{}.tar.gz",
        slugify(&audiobook.author),
        slugify(&audiobook.title),
        audiobook.date
    );

    Ok((filename, archive_content))
}
