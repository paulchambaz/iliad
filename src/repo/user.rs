use crate::{error::AppError, models::user::User};
use sqlx::SqlitePool;

pub async fn find_by_username(db: &SqlitePool, username: &str) -> Result<Option<User>, AppError> {
    sqlx::query_as!(
        User,
        r#"SELECT
            username as "username!",
            password_hash as "password_hash!"
        FROM users WHERE username = ?"#,
        username
    )
    .fetch_optional(db)
    .await
    .map_err(AppError::from)
}

pub async fn create(db: &SqlitePool, username: &str, password_hash: &str) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)",
        username,
        password_hash,
    )
    .execute(db)
    .await?;
    Ok(())
}
