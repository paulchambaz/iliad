mod config;
mod db;
mod middlewares;
mod models;
mod state;

use actix_web::middleware::from_fn;
use actix_web::{web, App, HttpResponse, HttpServer};
use config::Config;
use state::AppState;

async fn admin_route() -> HttpResponse {
    HttpResponse::Ok().body("Admin access granted")
}

async fn regular_route() -> HttpResponse {
    HttpResponse::Ok().body("Regular user access granted")
}

async fn public_route() -> HttpResponse {
    HttpResponse::Ok().body("Public access granted")
}

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
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(
                web::scope("/admin")
                    .wrap(from_fn(middlewares::auth::admin_auth))
                    .route("", web::get().to(admin_route)),
            )
            .service(
                web::scope("/user")
                    .wrap(from_fn(middlewares::auth::standard_auth))
                    .route("", web::get().to(regular_route)),
            )
            .route("/public", web::get().to(public_route))
    })
    .bind((config.server_address.clone(), config.server_port))?
    .run()
    .await
}
