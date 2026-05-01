use crate::{error::AppError, models::user::User};
use sqlx::SqlitePool;

pub async fn find_by_username(db: &SqlitePool, username: &str) -> Result<Option<User>, AppError> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(db)
        .await
        .map_err(AppError::from)
}

pub async fn create(db: &SqlitePool, username: &str, password_hash: &str) -> Result<(), AppError> {
    sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(username)
        .bind(password_hash)
        .execute(db)
        .await?;
    Ok(())
}
