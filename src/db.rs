use crate::error::AppError;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

pub async fn connect(db_path: &str) -> Result<SqlitePool, AppError> {
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::from_str(db_path)?.create_if_missing(true),
    )
    .await?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(pool)
}
