use anyhow::{Context, Result};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

pub async fn connect(db_path: &str) -> Result<SqlitePool> {
    SqlitePool::connect_with(SqliteConnectOptions::from_str(db_path)?.create_if_missing(true))
        .await
        .context("Failed to connect to database")
}

pub async fn init_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS audiobooks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            author TEXT NOT NULL,
            file_path TEXT NOT NULL UNIQUE,
            hash TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS positions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            audiobook_id INTEGER NOT NULL,
            position INTEGER NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (audiobook_id) REFERENCES audiobooks(id)
        );
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to initialize database schema")?;

    Ok(())
}
