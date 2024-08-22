use crate::{
    inputs::position::InputPositionUpdate, models::position::Position,
    outputs::position::OutputPositionUpdate, state::AppState,
};
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};

pub async fn get_position_user_book(
    username: String,
    book_hash: String,
    state: &AppState,
) -> Result<OutputPositionUpdate> {
    let position = sqlx::query_as::<_, Position>(
        "SELECT * FROM positions WHERE audiobook_hash = ? AND username = ?",
    )
    .bind(&book_hash)
    .bind(&username)
    .fetch_optional(&state.db)
    .await?;

    match position {
        Some(pos) => Ok(OutputPositionUpdate {
            chapter_index: pos.chapter_index,
            chapter_position: pos.chapter_position,
        }),
        None => Ok(OutputPositionUpdate {
            chapter_index: 0,
            chapter_position: 0,
        }),
    }
}

pub async fn put_position_user_book(
    username: String,
    book_hash: String,
    input: InputPositionUpdate,
    state: &AppState,
) -> Result<()> {
    let current_position = sqlx::query_as::<_, Position>(
        "SELECT * FROM positions WHERE audiobook_hash = ? AND username = ?",
    )
    .bind(&book_hash)
    .bind(&username)
    .fetch_optional(&state.db)
    .await?;

    let client_timestamp = DateTime::<Utc>::from_timestamp(input.client_timestamp, 0)
        .expect("Invalid timestamp")
        .naive_utc();

    match current_position {
        Some(pos) => {
            if should_update_position(&pos, &input, client_timestamp) {
                sqlx::query(
                    "UPDATE positions SET chapter_index = ?, chapter_position = ?, timestamp = ? WHERE audiobook_hash = ? AND username = ?"
                )
                .bind(input.chapter_index)
                .bind(input.chapter_position as i64)
                .bind(client_timestamp)
                .bind(&book_hash)
                .bind(&username)
                .execute(&state.db)
                .await?;
            }
        }
        None => {
            sqlx::query(
                "INSERT INTO positions (audiobook_hash, username, chapter_index, chapter_position, timestamp) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(&book_hash)
            .bind(&username)
            .bind(input.chapter_index)
            .bind(input.chapter_position as i64)
            .bind(client_timestamp)
            .execute(&state.db)
            .await?;
        }
    }

    Ok(())
}

fn should_update_position(
    current: &Position,
    input: &InputPositionUpdate,
    client_timestamp: NaiveDateTime,
) -> bool {
    // If the input is more recent and further in the book, always update
    if client_timestamp > current.timestamp
        && (input.chapter_index > current.chapter_index
            || (input.chapter_index == current.chapter_index
                && input.chapter_position > current.chapter_position))
    {
        return true;
    }

    // If the input is older but significantly further in the book, update
    if client_timestamp <= current.timestamp
        && (input.chapter_index > current.chapter_index + 1
            || (input.chapter_index == current.chapter_index + 1
                && input.chapter_position > current.chapter_position + 300000))
    {
        // 5 minutes further
        return true;
    }

    // If the input is very recent (within last 10 minutes) and not too far back, update
    if client_timestamp
        > current
            .timestamp
            .checked_sub_signed(chrono::Duration::minutes(10))
            .unwrap_or(current.timestamp)
        && (input.chapter_index >= current.chapter_index.saturating_sub(1))
    {
        return true;
    }

    false
}
