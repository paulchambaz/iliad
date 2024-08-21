mod config;
mod db;
mod handlers;
mod inputs;
mod middlewares;
mod models;
mod outputs;
mod services;
mod state;

use actix_web::middleware::from_fn;
use actix_web::{web, App, HttpServer};
use config::Config;
use handlers::audiobook::{get_audiobook, get_audiobook_download, get_audiobooks};
use handlers::auth::{post_auth_admin, post_auth_login, post_auth_register};
use handlers::library::{put_library_cleanup, put_library_scan};
use handlers::position::{get_position, put_position};
use middlewares::auth::{admin_auth, standard_auth};
use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::new();
    let state = AppState::new(&config)
        .await
        .expect("Failed to initialize application state");

    println!(
        "Starting server at {}:{}",
        config.server_address, config.server_port
    );

    HttpServer::new(move || {
        let mut app = App::new().app_data(web::Data::new(state.clone()));

        // auth routes
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

        // audiobook routes
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

        // position routes
        app = app.service(
            web::resource("/positions/{hash}")
                .wrap(from_fn(standard_auth))
                .route(web::get().to(get_position))
                .route(web::put().to(put_position)),
        );

        // library routes
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
    .bind((config.server_address.clone(), config.server_port))?
    .run()
    .await
}
