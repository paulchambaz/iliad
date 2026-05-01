use crate::{error::AppError, models::audiobook::Audiobook};
use sqlx::SqlitePool;
use std::collections::HashMap;

pub async fn find_all(db: &SqlitePool) -> Result<Vec<Audiobook>, AppError> {
    sqlx::query_as::<_, Audiobook>("SELECT * FROM audiobooks")
        .fetch_all(db)
        .await
        .map_err(AppError::from)
}

pub async fn find_by_hash(db: &SqlitePool, hash: &str) -> Result<Option<Audiobook>, AppError> {
    sqlx::query_as::<_, Audiobook>("SELECT * FROM audiobooks WHERE hash = ?")
        .bind(hash)
        .fetch_optional(db)
        .await
        .map_err(AppError::from)
}

pub async fn find_hashes_with_checksums(
    db: &SqlitePool,
) -> Result<HashMap<String, Option<String>>, AppError> {
    let rows: Vec<(String, Option<String>)> =
        sqlx::query_as("SELECT hash, archive_checksum FROM audiobooks")
            .fetch_all(db)
            .await?;
    Ok(rows.into_iter().collect())
}

pub async fn find_path(db: &SqlitePool, hash: &str) -> Result<Option<String>, AppError> {
    sqlx::query_scalar("SELECT path FROM audiobooks WHERE hash = ?")
        .bind(hash)
        .fetch_optional(db)
        .await
        .map_err(AppError::from)
}

pub async fn create(db: &SqlitePool, audiobook: &Audiobook) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO audiobooks (hash, title, author, date, description, genres, duration, size, path, final_chapter_index, final_chapter_position) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&audiobook.hash)
    .bind(&audiobook.title)
    .bind(&audiobook.author)
    .bind(audiobook.date)
    .bind(&audiobook.description)
    .bind(&audiobook.genres)
    .bind(audiobook.duration)
    .bind(audiobook.size)
    .bind(&audiobook.path)
    .bind(audiobook.final_chapter_index)
    .bind(audiobook.final_chapter_position)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn update(db: &SqlitePool, audiobook: &Audiobook) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE audiobooks SET title = ?, author = ?, date = ?, description = ?, genres = ?, duration = ?, size = ?, path = ?, final_chapter_index = ?, final_chapter_position = ? WHERE hash = ?",
    )
    .bind(&audiobook.title)
    .bind(&audiobook.author)
    .bind(audiobook.date)
    .bind(&audiobook.description)
    .bind(&audiobook.genres)
    .bind(audiobook.duration)
    .bind(audiobook.size)
    .bind(&audiobook.path)
    .bind(audiobook.final_chapter_index)
    .bind(audiobook.final_chapter_position)
    .bind(&audiobook.hash)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn delete(db: &SqlitePool, hash: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM audiobooks WHERE hash = ?")
        .bind(hash)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn mark_ready(db: &SqlitePool, hash: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE audiobooks SET archive_ready = 1 WHERE hash = ?")
        .bind(hash)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn mark_pending(db: &SqlitePool, hash: &str, checksum: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE audiobooks SET archive_ready = 0, archive_checksum = ? WHERE hash = ?")
        .bind(checksum)
        .bind(hash)
        .execute(db)
        .await?;
    Ok(())
}
