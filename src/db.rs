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
            username TEXT PRIMARY KEY,
            password_hash TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS audiobooks (
            hash TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            author TEXT NOT NULL,
            date INTEGER NOT NULL,
            description TEXT NOT NULL,
            genres TEXT NOT NULL,
            duration INTEGER NOT NULL,
            size INTEGER NOT NULL,
            path TEXT NOT NULL UNIQUE,
            final_chapter_index INTEGER,
            final_chapter_position INTEGER
        );

        CREATE TABLE IF NOT EXISTS positions (
            audiobook_hash TEXT NOT NULL,
            username TEXT NOT NULL,
            chapter_index INTEGER NOT NULL,
            chapter_position INTEGER NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (audiobook_hash, username)
        );
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to initialize database schema")?;

    Ok(())
}
