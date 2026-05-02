use crate::{
    error::AppError,
    inputs::position::InputPositionUpdate,
    outputs::position::OutputPositionUpdate,
    repo::position as position_repo,
    state::AppState,
};
use chrono::{DateTime, Utc};

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
            timestamp: pos.timestamp.and_utc().timestamp(),
        }),
        None => Ok(OutputPositionUpdate {
            chapter_index: 0,
            chapter_position: 0,
            timestamp: 0,
        }),
    }
}

pub async fn put_position_user_book(
    username: String,
    book_hash: String,
    input: InputPositionUpdate,
    state: &AppState,
) -> Result<(), AppError> {
    let timestamp = DateTime::<Utc>::from_timestamp(input.timestamp, 0)
        .ok_or_else(|| AppError::Internal("invalid client timestamp".into()))?
        .naive_utc();

    let current_position = position_repo::find(&state.db, &book_hash, &username).await?;

    match current_position {
        Some(_) => {
            position_repo::update(
                &state.db,
                &book_hash,
                &username,
                input.chapter_index,
                input.chapter_position,
                timestamp,
            )
            .await?;
        }
        None => {
            position_repo::insert(
                &state.db,
                &book_hash,
                &username,
                input.chapter_index,
                input.chapter_position,
                timestamp,
            )
            .await?;
        }
    }

    Ok(())
}

