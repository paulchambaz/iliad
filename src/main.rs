mod config;
mod db;
mod error;
mod handlers;
mod inputs;
mod middlewares;
mod models;
mod outputs;
mod repo;
mod services;
mod state;

use actix_web::middleware::from_fn;
use actix_web::{web, App, HttpServer};
use config::Config;
use error::AppError;
use handlers::audiobook::{get_audiobook, get_audiobook_download, get_audiobooks};
use handlers::auth::{post_auth_admin, post_auth_login, post_auth_register};
use handlers::library::{put_library_cleanup, put_library_scan};
use handlers::position::{get_position, put_position};
use middlewares::auth::{admin_auth, standard_auth};
use middlewares::logging::log_request;
use repo::audiobook as audiobook_repo;
use services::library::{build_archive, scan_library};
use state::AppState;
use std::path::PathBuf;

#[actix_web::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;
    let state = AppState::new(&config).await?;

    let worker_state = state.clone();
    tokio::spawn(async move {
        loop {
            loop {
                let hash = {
                    let mut q = worker_state.archive_queue.lock().unwrap();
                    match q.pending.pop_front() {
                        Some(h) => {
                            q.in_progress = Some(h.clone());
                            h
                        }
                        None => {
                            q.in_progress = None;
                            break;
                        }
                    }
                };

                let path = audiobook_repo::find_path(&worker_state.db, &hash)
                    .await
                    .ok()
                    .flatten();

                if let Some(path) = path {
                    let dir = PathBuf::from(path);
                    let result = tokio::task::spawn_blocking(move || build_archive(&dir)).await;
                    match result {
                        Ok(Ok(())) => {
                            let _ = audiobook_repo::mark_ready(&worker_state.db, &hash).await;
                            tracing::info!("archive ready: {}", hash);
                        }
                        Ok(Err(e)) => tracing::error!("archive failed {}: {}", hash, e),
                        Err(e) => tracing::error!("archive task panicked {}: {}", hash, e),
                    }
                }

                worker_state.archive_queue.lock().unwrap().in_progress = None;
            }
            worker_state.archive_notify.notified().await;
        }
    });

    if let Err(e) = scan_library(&state).await {
        tracing::error!("initial library scan failed: {}", e);
        std::process::exit(1);
    }

    tracing::info!("starting server at {}:{}", config.server_address, config.server_port);

    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(from_fn(log_request));

        app = app.service(web::resource("/auth/login").route(web::post().to(post_auth_login)));
        app = app.service(web::resource("/auth/admin").route(web::post().to(post_auth_admin)));
        if config.public_register {
            app = app
                .service(web::resource("/auth/register").route(web::post().to(post_auth_register)));
        } else {
            app = app.service(
                web::resource("/auth/register")
                    .wrap(from_fn(admin_auth))
                    .route(web::post().to(post_auth_register)),
            );
        }

        app = app.service(
            web::resource("/audiobooks")
                .wrap(from_fn(standard_auth))
                .route(web::get().to(get_audiobooks)),
        );
        app = app.service(
            web::resource("/audiobooks/{hash}")
                .wrap(from_fn(standard_auth))
                .route(web::get().to(get_audiobook)),
        );
        app = app.service(
            web::resource("/audiobooks/{hash}/download")
                .wrap(from_fn(standard_auth))
                .route(web::get().to(get_audiobook_download)),
        );

        app = app.service(
            web::resource("/positions/{hash}")
                .wrap(from_fn(standard_auth))
                .route(web::get().to(get_position))
                .route(web::put().to(put_position)),
        );

        app = app.service(
            web::resource("/library/scan")
                .wrap(from_fn(admin_auth))
                .route(web::put().to(put_library_scan)),
        );
        app = app.service(
            web::resource("/library/cleanup")
                .wrap(from_fn(admin_auth))
                .route(web::put().to(put_library_cleanup)),
        );

        app
    })
    .bind((config.server_address.as_str(), config.server_port))?
    .run()
    .await?;

    Ok(())
}
