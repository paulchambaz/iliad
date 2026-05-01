use crate::{error::AppError, models::audiobook::Audiobook};
use sqlx::SqlitePool;
use std::collections::HashMap;

pub async fn find_all(db: &SqlitePool) -> Result<Vec<Audiobook>, AppError> {
    sqlx::query_as!(
        Audiobook,
        r#"SELECT
            hash as "hash!",
            title as "title!",
            author as "author!",
            date as "date!: i32",
            description as "description!",
            genres as "genres!",
            duration as "duration!",
            size as "size!",
            path as "path!",
            final_chapter_index as "final_chapter_index!",
            final_chapter_position as "final_chapter_position!",
            cover,
            archive_ready as "archive_ready!: bool"
        FROM audiobooks"#
    )
    .fetch_all(db)
    .await
    .map_err(AppError::from)
}

pub async fn find_by_hash(db: &SqlitePool, hash: &str) -> Result<Option<Audiobook>, AppError> {
    sqlx::query_as!(
        Audiobook,
        r#"SELECT
            hash as "hash!",
            title as "title!",
            author as "author!",
            date as "date!: i32",
            description as "description!",
            genres as "genres!",
            duration as "duration!",
            size as "size!",
            path as "path!",
            final_chapter_index as "final_chapter_index!",
            final_chapter_position as "final_chapter_position!",
            cover,
            archive_ready as "archive_ready!: bool"
        FROM audiobooks WHERE hash = ?"#,
        hash
    )
    .fetch_optional(db)
    .await
    .map_err(AppError::from)
}

pub async fn find_hashes_with_checksums(
    db: &SqlitePool,
) -> Result<HashMap<String, (Option<String>, bool)>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT
            hash as "hash!",
            archive_checksum,
            archive_ready as "archive_ready!: bool"
        FROM audiobooks"#
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| (r.hash, (r.archive_checksum, r.archive_ready)))
        .collect())
}

pub async fn find_path(db: &SqlitePool, hash: &str) -> Result<Option<String>, AppError> {
    sqlx::query_scalar!("SELECT path FROM audiobooks WHERE hash = ?", hash)
        .fetch_optional(db)
        .await
        .map_err(AppError::from)
}

pub async fn create(db: &SqlitePool, audiobook: &Audiobook) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO audiobooks (hash, title, author, date, description, genres, duration, size, path, final_chapter_index, final_chapter_position, cover) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        audiobook.hash,
        audiobook.title,
        audiobook.author,
        audiobook.date,
        audiobook.description,
        audiobook.genres,
        audiobook.duration,
        audiobook.size,
        audiobook.path,
        audiobook.final_chapter_index,
        audiobook.final_chapter_position,
        audiobook.cover,
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn update(db: &SqlitePool, audiobook: &Audiobook) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE audiobooks SET title = ?, author = ?, date = ?, description = ?, genres = ?, duration = ?, size = ?, path = ?, final_chapter_index = ?, final_chapter_position = ?, cover = ? WHERE hash = ?",
        audiobook.title,
        audiobook.author,
        audiobook.date,
        audiobook.description,
        audiobook.genres,
        audiobook.duration,
        audiobook.size,
        audiobook.path,
        audiobook.final_chapter_index,
        audiobook.final_chapter_position,
        audiobook.cover,
        audiobook.hash,
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn delete(db: &SqlitePool, hash: &str) -> Result<(), AppError> {
    sqlx::query!("DELETE FROM audiobooks WHERE hash = ?", hash)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn mark_ready(db: &SqlitePool, hash: &str) -> Result<(), AppError> {
    sqlx::query!("UPDATE audiobooks SET archive_ready = 1 WHERE hash = ?", hash)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn mark_pending(db: &SqlitePool, hash: &str, checksum: &str) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE audiobooks SET archive_ready = 0, archive_checksum = ? WHERE hash = ?",
        checksum,
        hash,
    )
    .execute(db)
    .await?;
    Ok(())
}
