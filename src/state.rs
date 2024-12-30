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
        if let Some(db_dir) = PathBuf::from(&config.db_path).parent() {
            fs::create_dir_all(db_dir).context("Failed to create database directory")?;
        }

        let library_path = PathBuf::from(&config.library_path);
        fs::create_dir_all(&library_path).context("Failed to create library directory")?;

        let db = db::connect(&config.db_path).await?;
        db::init_schema(&db).await?;

        Ok(Self {
            db,
            library_path,
            admin_password: config.admin_password.clone(),
            regular_tokens: Arc::new(Mutex::new(HashMap::new())),
            admin_tokens: Arc::new(Mutex::new(Vec::new())),
        })
    }
}
