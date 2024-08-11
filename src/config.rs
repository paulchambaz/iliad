use dotenv::dotenv;
use std::env;
use std::path::PathBuf;

pub struct Config {
    pub db_path: PathBuf,
    pub library_path: PathBuf,
    pub scan_interval: u64,
    pub cleanup_interval: u64,
    pub server_address: String,
    pub server_port: u16,
    pub allow_register: bool,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        Self {
            db_path: env::var("ILIAD_DB_PATH")
                .unwrap_or_else(|_| "/app/instance/iliad.db".to_string())
                .into(),
            library_path: env::var("ILIAD_LIBRARY_PATH")
                .unwrap_or_else(|_| "/app/instance/library".to_string())
                .into(),
            scan_interval: env::var("ILIAD_SCAN_INTERVAL")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("Invalid ILIAD_SCAN_INTERVAL"),
            cleanup_interval: env::var("ILIAD_CLEANUP_INTERVAL")
                .unwrap_or_else(|_| "365".to_string())
                .parse()
                .expect("Invalid ILIAD_CLEANUP_INTERVAL"),
            server_address: env::var("ILIAD_SERVER_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("ILIAD_SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("Invalid ILIAD_SERVER_PORT"),
            allow_register: env::var("ILIAD_ALLOW_REGISTER")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .expect("Invalid ILIAD_ALLOW_REGISTER"),
        }
    }
}
