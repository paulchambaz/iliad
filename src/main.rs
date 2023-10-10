#![feature(proc_macro_hygiene, decl_macro)]

extern crate diesel;

use diesel::RunQueryDsl;
use dotenv::dotenv;
use env_logger;
use log::{error, info, warn, LevelFilter};
use path_abs::PathAbs;
use std::net::IpAddr;
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::{env, process::exit};

mod controllers;
mod database;
mod errors;
mod models;
mod routes;
mod scan;
mod schema;

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    ensure_migrations();

    let (scan_interval, server_address, server_port, allow_register) = check_env();

    periodic_task(
        || match scan::scan_audiobooks_directory() {
            Ok(_) => info!("Audiobook scan completed"),
            Err(_) => warn!("Audiobook scan failed"),
        },
        scan_interval * 60,
    );

    periodic_task(
        || match database::cleanup_positions() {
            Ok(_) => info!("Positions cleanup completed"),
            Err(_) => warn!("Position cleanup failed"),
        },
        86400,
    );

    let rocket = routes::create_rocket(server_address, server_port, allow_register);
    rocket.launch();
}

/// Periodically executes a given task with the specified duration. This funciton is used for
/// background tasks and scheduled jobs that need to run at regular intervals.
///
/// # Paramaters
///
/// * `task` - The function or closure to be executed periodically
/// * `duration_secs`: The interval (in seconds) between each execution of the task
fn periodic_task<F: Fn() + Send + 'static>(task: F, duration_secs: u64) {
    thread::spawn(move || loop {
        task();
        thread::sleep(Duration::from_secs(duration_secs));
    });
}

/// Validates and fetches environment variables required for the program's configuration. Exits the
/// program on failure.
///
/// # Returns
///
/// A tuple containing : (scan_interval, server_address, server_port, allow_register)
fn check_env() -> (u64, IpAddr, u16, bool) {
    dotenv().ok();

    let check_var = |key: &str| -> String {
        match env::var(key) {
            Ok(value) if !value.is_empty() => value,
            _ => {
                error!("{} not set or empty but required", key);
                exit(1);
            }
        }
    };

    let db_path_str = check_var("ILIAD_DB_PATH");
    let db_path = PathAbs::new(db_path_str).unwrap_or_else(|_| {
        error!("Failed to convert ILIAD_DB_PATH to an absolute path");
        exit(1);
    });

    let underlying_path = db_path.as_path();

    if let Some(parent) = underlying_path.parent() {
        if !parent.exists() || !parent.is_dir() {
            error!("Parent directory of ILIAD_DB_PATH does not exist or is not a directory");
            exit(1);
        }
    } else {
        error!("ILIAD_DB_PATH does not have a valid parent directory");
        exit(1);
    }

    let library_path = check_var("ILIAD_LIBRARY_PATH");
    let library_path = Path::new(&library_path);
    if !library_path.exists() || !library_path.is_dir() {
        error!("ILIAD_LIBRARY_PATH does not exist or is not a directory");
        exit(1);
    }

    let scan_interval: u64 = check_var("ILIAD_SCAN_INTERVAL")
        .parse()
        .unwrap_or_else(|_| {
            error!("ILIAD_SCAN_INTERVAL should be a positive integer");
            exit(1);
        });
    if scan_interval <= 0 {
        error!("ILIAD_SCAN_INTERVAL should be a positive integer");
        exit(1);
    }

    let cleanup_interval: i64 = check_var("ILIAD_CLEANUP_INTERVAL")
        .parse()
        .unwrap_or_else(|_| {
            error!("ILIAD_CLEANUP_INTERVAL should be a positive integer");
            exit(1);
        });
    if cleanup_interval <= 0 {
        error!("ILIAD_CLEANUP_INTERVAL should be a positive integer");
        exit(1);
    }

    let server_port: u16 = check_var("ILIAD_SERVER_PORT").parse().unwrap_or_else(|_| {
        error!("ILIAD_SERVER_PORT should be a valid port between 1 and 65535");
        exit(1);
    });
    if server_port <= 0 {
        error!("ILIAD_SERVER_PORT should be a valid port between 1 and 65535");
        exit(1);
    }

    let server_address_str = check_var("ILIAD_SERVER_ADDRESS");
    let server_address = match server_address_str.parse::<IpAddr>() {
        Ok(address) => address,
        Err(_) => {
            error!("ILIAD_SERVER_ADDRESS should be a valid IPv4 or IPv6 address");
            exit(1);
        }
    };

    let allow_register: bool = match check_var("ILIAD_ALLOW_REGISTER").as_str() {
        "true" => true,
        "false" => false,
        _ => {
            error!("ILIAD_ALLOW_REGISTER should be either true or false");
            exit(1);
        }
    };

    (scan_interval, server_address, server_port, allow_register)
}

/// Ensures that the necessary database migrations for the application are executed. Initializes
/// the database tables if they do not already exist.
///
/// Creates the following tables:
/// - `accounts`: To store user credentials and related data.
/// - `positions`: To store the position in a media file for a user.
/// - `audiobooks`: To store details about available audiobooks.
pub fn ensure_migrations() {
    let conn = &mut match database::establish_connection() {
        Ok(conn) => conn,
        Err(_) => panic!("Failed to ensure migration"),
    };

    let query_accounts = r#"
    CREATE TABLE IF NOT EXISTS "accounts" (
      "username" TEXT NOT NULL,
      "password" TEXT NOT NULL,
      "key"      TEXT NOT NULL,
      PRIMARY KEY ("username")
    );"#;

    let query_positions = r#"
    CREATE TABLE IF NOT EXISTS "positions" (
      "hash"          TEXT NOT NULL,
      "username"      TEXT NOT NULL,
      "file"          TEXT NOT NULL,
      "position"      INTEGER NOT NULL,
      "last_modified" DATE NOT NULL,
      PRIMARY KEY("hash", "username")
    );"#;

    let query_audiobooks = r#"
    CREATE TABLE IF NOT EXISTS "audiobooks" (
      "hash"   TEXT NOT NULL,
      "title"  TEXT NOT NULL,
      "author" TEXT NOT NULL,
      "path"   TEXT NOT NULL,
      PRIMARY KEY ("hash")
    );"#;

    diesel::sql_query(query_accounts)
        .execute(conn)
        .expect("Failed to create account table");
    diesel::sql_query(query_positions)
        .execute(conn)
        .expect("Failed to create account table");
    diesel::sql_query(query_audiobooks)
        .execute(conn)
        .expect("Failed to create account table");
}
