use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::db;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub library_path: PathBuf,
    pub admin_password: String,

    pub regular_tokens: Arc<Mutex<HashMap<String, String>>>,
    pub admin_tokens: Arc<Mutex<Vec<String>>>,
}

impl AppState {
    pub async fn new(config: &crate::config::Config) -> Result<Self> {
        // Ensure parent directories exist
        if let Some(db_dir) = PathBuf::from(&config.db_path).parent() {
            fs::create_dir_all(db_dir).context("Failed to create database directory")?;
        }

        let library_path = PathBuf::from(&config.library_path);
        fs::create_dir_all(&library_path).context("Failed to create library directory")?;

        // Connect to the database and initialize schema
        let db = db::connect(&config.db_path).await?;
        db::init_schema(&db).await?;

        let mut regular_tokens = HashMap::new();
        regular_tokens.insert("tok1".to_string(), "user1".to_string());
        regular_tokens.insert("tok2".to_string(), "user2".to_string());
        regular_tokens.insert("tok3".to_string(), "user3".to_string());

        let admin_tokens = vec!["adm1".to_string(), "adm2".to_string()];

        Ok(Self {
            db,
            library_path,
            admin_password: config.admin_password.clone(),
            regular_tokens: Arc::new(Mutex::new(regular_tokens)),
            admin_tokens: Arc::new(Mutex::new(admin_tokens)),
        })
    }
}
