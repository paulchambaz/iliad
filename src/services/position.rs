use crate::{
    error::AppError,
    inputs::position::InputPositionUpdate,
    models::position::Position,
    outputs::position::OutputPositionUpdate,
    repo::position as position_repo,
    state::AppState,
};
use chrono::{DateTime, NaiveDateTime, Utc};

pub async fn get_position_user_book(
    username: String,
    book_hash: String,
    state: &AppState,
) -> Result<OutputPositionUpdate, AppError> {
    let position = position_repo::find(&state.db, &book_hash, &username).await?;

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
) -> Result<(), AppError> {
    let current_position = position_repo::find(&state.db, &book_hash, &username).await?;

    let client_timestamp = DateTime::<Utc>::from_timestamp(input.client_timestamp, 0)
        .ok_or_else(|| AppError::Internal("invalid client timestamp".into()))?
        .naive_utc();

    match current_position {
        Some(pos) => {
            if should_update_position(&pos, &input, client_timestamp) {
                position_repo::update(
                    &state.db,
                    &book_hash,
                    &username,
                    input.chapter_index,
                    input.chapter_position,
                    client_timestamp,
                )
                .await?;
            }
        }
        None => {
            position_repo::insert(
                &state.db,
                &book_hash,
                &username,
                input.chapter_index,
                input.chapter_position,
                client_timestamp,
            )
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
    if client_timestamp > current.timestamp
        && (input.chapter_index > current.chapter_index
            || (input.chapter_index == current.chapter_index
                && input.chapter_position > current.chapter_position))
    {
        return true;
    }

    if client_timestamp <= current.timestamp
        && (input.chapter_index > current.chapter_index + 1
            || (input.chapter_index == current.chapter_index + 1
                && input.chapter_position > current.chapter_position + 300000))
    {
        return true;
    }

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
