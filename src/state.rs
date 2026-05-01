use crate::{db, error::AppError};
use sqlx::SqlitePool;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Notify;

pub struct ArchiveQueue {
    pub pending: VecDeque<String>,
    pub in_progress: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub library_path: PathBuf,
    pub admin_password: String,
    pub token_ttl: Duration,

    pub regular_tokens: Arc<Mutex<HashMap<String, (String, Instant)>>>,
    pub admin_tokens: Arc<Mutex<Vec<(String, Instant)>>>,
    pub archive_queue: Arc<Mutex<ArchiveQueue>>,
    pub archive_notify: Arc<Notify>,
}

impl AppState {
    pub async fn new(config: &crate::config::Config) -> Result<Self, AppError> {
        if let Some(db_dir) = PathBuf::from(&config.db_path).parent() {
            fs::create_dir_all(db_dir)?;
        }

        let library_path = PathBuf::from(&config.library_path);
        fs::create_dir_all(&library_path)?;

        let db = db::connect(&config.db_path).await?;

        Ok(Self {
            db,
            library_path,
            admin_password: config.admin_password.clone(),
            token_ttl: Duration::from_secs(config.token_ttl_hours * 3600),
            regular_tokens: Arc::new(Mutex::new(HashMap::new())),
            admin_tokens: Arc::new(Mutex::new(Vec::new())),
            archive_queue: Arc::new(Mutex::new(ArchiveQueue {
                pending: VecDeque::new(),
                in_progress: None,
            })),
            archive_notify: Arc::new(Notify::new()),
        })
    }
}
