use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub db_path: String,
    pub library_path: String,
    pub server_address: String,
    pub server_port: u16,
    pub public_register: bool,
    pub admin_password: String,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();

        Config {
            db_path: env::var("ILIAD_DB_PATH")
                .unwrap_or_else(|_| "/app/instance/iliad.db".to_string()),
            library_path: env::var("ILIAD_LIBRARY_PATH")
                .unwrap_or_else(|_| "/app/instance/library".to_string()),
            server_address: env::var("ILIAD_SERVER_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("ILIAD_SERVER_PORT")
                .unwrap_or_else(|_| "9090".to_string())
                .parse()
                .unwrap(),
            public_register: env::var("ILIAD_PUBLIC_REGISTER")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap(),
            admin_password: env::var("ILIAD_ADMIN_PASSWORD")
                .expect("ILIAD_ADMIN_PASSWORD must be set"),
        }
    }
}
