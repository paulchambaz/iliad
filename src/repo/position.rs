use crate::{error::AppError, models::position::Position};
use chrono::NaiveDateTime;
use sqlx::SqlitePool;

pub async fn find(
    db: &SqlitePool,
    audiobook_hash: &str,
    username: &str,
) -> Result<Option<Position>, AppError> {
    sqlx::query_as!(
        Position,
        r#"SELECT
            audiobook_hash as "audiobook_hash!",
            username as "username!",
            chapter_index as "chapter_index!",
            chapter_position as "chapter_position!",
            timestamp as "timestamp!: NaiveDateTime"
        FROM positions WHERE audiobook_hash = ? AND username = ?"#,
        audiobook_hash,
        username,
    )
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
    sqlx::query!(
        "INSERT INTO positions (audiobook_hash, username, chapter_index, chapter_position, timestamp) VALUES (?, ?, ?, ?, ?)",
        audiobook_hash,
        username,
        chapter_index,
        chapter_position,
        timestamp,
    )
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
    sqlx::query!(
        "UPDATE positions SET chapter_index = ?, chapter_position = ?, timestamp = ? WHERE audiobook_hash = ? AND username = ?",
        chapter_index,
        chapter_position,
        timestamp,
        audiobook_hash,
        username,
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn delete_old(db: &SqlitePool, cutoff: NaiveDateTime) -> Result<(), AppError> {
    sqlx::query!("DELETE FROM positions WHERE timestamp < ?", cutoff)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn delete_beyond_final(db: &SqlitePool) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM positions
         WHERE EXISTS (
             SELECT 1 FROM audiobooks
             WHERE audiobooks.hash = positions.audiobook_hash
             AND (positions.chapter_index > audiobooks.final_chapter_index
                  OR (positions.chapter_index = audiobooks.final_chapter_index
                      AND positions.chapter_position >= audiobooks.final_chapter_position))
         )"
    )
    .execute(db)
    .await?;
    Ok(())
}
