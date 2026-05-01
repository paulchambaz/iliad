use crate::error::AppError;
use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub db_path: String,
    pub library_path: String,
    pub server_address: String,
    pub server_port: u16,
    pub public_register: bool,
    pub admin_password: String,
    pub token_ttl_hours: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        dotenv().ok();

        let server_port = env::var("ILIAD_SERVER_PORT")
            .unwrap_or_else(|_| "9090".to_string())
            .parse::<u16>()
            .map_err(|e| AppError::Internal(format!("invalid ILIAD_SERVER_PORT: {e}")))?;

        let public_register = env::var("ILIAD_PUBLIC_REGISTER")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .map_err(|e| AppError::Internal(format!("invalid ILIAD_PUBLIC_REGISTER: {e}")))?;

        let admin_password = env::var("ILIAD_ADMIN_PASSWORD")
            .map_err(|_| AppError::Internal("ILIAD_ADMIN_PASSWORD must be set".to_string()))?;

        let token_ttl_hours = env::var("ILIAD_TOKEN_TTL_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<u64>()
            .map_err(|e| AppError::Internal(format!("invalid ILIAD_TOKEN_TTL_HOURS: {e}")))?;

        Ok(Config {
            db_path: env::var("ILIAD_DB_PATH")
                .unwrap_or_else(|_| "/app/instance/iliad.db".to_string()),
            library_path: env::var("ILIAD_LIBRARY_PATH")
                .unwrap_or_else(|_| "/app/instance/library".to_string()),
            server_address: env::var("ILIAD_SERVER_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port,
            public_register,
            admin_password,
            token_ttl_hours,
        })
    }
}
