use crate::config::Config;
use crate::error::IliadError;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub async fn establish_connection(config: &Config) -> Result<SqlitePool, IliadError> {
    let database_url = format!("sqlite:{}", config.db_path.display());
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| IliadError::DatabaseError(e.to_string()))
}
