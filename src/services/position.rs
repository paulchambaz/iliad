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
    let client_timestamp = DateTime::<Utc>::from_timestamp(input.client_timestamp, 0)
        .ok_or_else(|| AppError::Internal("invalid client timestamp".into()))?
        .naive_utc();

    let now = Utc::now().naive_utc();
    if client_timestamp > now + chrono::Duration::seconds(300) {
        return Err(AppError::BadRequest(
            "client_timestamp is too far in the future".into(),
        ));
    }

    let current_position = position_repo::find(&state.db, &book_hash, &username).await?;

    match current_position {
        Some(pos) => {
            if should_update_position(&pos, client_timestamp) {
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

fn should_update_position(current: &Position, client_timestamp: NaiveDateTime) -> bool {
    client_timestamp > current.timestamp
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::AppError,
        inputs::position::InputPositionUpdate,
        state::{AppState, ArchiveQueue},
    };
    use chrono::Utc;
    use sqlx::SqlitePool;
    use std::collections::{HashMap, VecDeque};
    use std::sync::{Arc, Mutex};

    fn make_position(timestamp: NaiveDateTime) -> Position {
        Position {
            audiobook_hash: "hash".into(),
            username: "user".into(),
            chapter_index: 0,
            chapter_position: 0,
            timestamp,
        }
    }

    #[test]
    fn newer_timestamp_wins() {
        let now = Utc::now().naive_utc();
        let current = make_position(now);
        let client_ts = now + chrono::Duration::seconds(10);
        assert!(should_update_position(&current, client_ts));
    }

    #[test]
    fn older_discarded() {
        let now = Utc::now().naive_utc();
        let current = make_position(now);
        let client_ts = now - chrono::Duration::seconds(10);
        assert!(!should_update_position(&current, client_ts));
    }

    #[test]
    fn equal_discarded() {
        let now = Utc::now().naive_utc();
        let current = make_position(now);
        assert!(!should_update_position(&current, now));
    }

    #[tokio::test]
    async fn future_timestamp_returns_400() {
        let db = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&db).await.unwrap();
        let state = AppState {
            db,
            library_path: std::path::PathBuf::from("/tmp"),
            admin_password: "test".into(),
            token_ttl: std::time::Duration::from_secs(3600),
            regular_tokens: Arc::new(Mutex::new(HashMap::new())),
            admin_tokens: Arc::new(Mutex::new(Vec::new())),
            archive_queue: Arc::new(Mutex::new(ArchiveQueue {
                pending: VecDeque::new(),
                in_progress: None,
            })),
            archive_notify: Arc::new(tokio::sync::Notify::new()),
        };
        let input = InputPositionUpdate {
            chapter_index: 0,
            chapter_position: 0,
            client_timestamp: Utc::now().timestamp() + 400,
        };
        let result = put_position_user_book("user".into(), "hash".into(), input, &state).await;
        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }
}
