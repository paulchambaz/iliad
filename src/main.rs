mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod services;
mod utils;

use actix_web::{App, HttpServer};
use config::Config;
use db::establish_connection;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env();
    let _pool = establish_connection(&config)
        .await
        .expect("Failed to connect to database");

    HttpServer::new(move || {
        App::new()
        // TODO: Add middleware
        // TODO: Configure services
        // TODO: Set up routes
    })
    .bind((config.server_address, config.server_port))?
    .run()
    .await
}
