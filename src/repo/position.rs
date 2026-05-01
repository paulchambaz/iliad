use crate::{error::AppError, models::position::Position};
use chrono::NaiveDateTime;
use sqlx::SqlitePool;

pub async fn find(
    db: &SqlitePool,
    audiobook_hash: &str,
    username: &str,
) -> Result<Option<Position>, AppError> {
    sqlx::query_as::<_, Position>(
        "SELECT * FROM positions WHERE audiobook_hash = ? AND username = ?",
    )
    .bind(audiobook_hash)
    .bind(username)
    .fetch_optional(db)
    .await
    .map_err(AppError::from)
}

pub async fn insert(
    db: &SqlitePool,
    audiobook_hash: &str,
    username: &str,
    chapter_index: i64,
    chapter_position: i64,
    timestamp: NaiveDateTime,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO positions (audiobook_hash, username, chapter_index, chapter_position, timestamp) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(audiobook_hash)
    .bind(username)
    .bind(chapter_index)
    .bind(chapter_position)
    .bind(timestamp)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn update(
    db: &SqlitePool,
    audiobook_hash: &str,
    username: &str,
    chapter_index: i64,
    chapter_position: i64,
    timestamp: NaiveDateTime,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE positions SET chapter_index = ?, chapter_position = ?, timestamp = ? WHERE audiobook_hash = ? AND username = ?",
    )
    .bind(chapter_index)
    .bind(chapter_position)
    .bind(timestamp)
    .bind(audiobook_hash)
    .bind(username)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn delete_old(db: &SqlitePool, cutoff: NaiveDateTime) -> Result<(), AppError> {
    sqlx::query("DELETE FROM positions WHERE timestamp < ?")
        .bind(cutoff)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn delete_beyond_final(db: &SqlitePool) -> Result<(), AppError> {
    sqlx::query(
        "DELETE FROM positions
         WHERE EXISTS (
             SELECT 1 FROM audiobooks
             WHERE audiobooks.hash = positions.audiobook_hash
             AND (positions.chapter_index > audiobooks.final_chapter_index
                  OR (positions.chapter_index = audiobooks.final_chapter_index
                      AND positions.chapter_position >= audiobooks.final_chapter_position))
         )",
    )
    .execute(db)
    .await?;
    Ok(())
}
