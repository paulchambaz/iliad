use actix_web::{web, HttpResponse};

use crate::state::AppState;

pub async fn get_audiobooks(state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().body("Get audiobooks route")
}

pub async fn get_audiobook(state: web::Data<AppState>, path: web::Path<String>) -> HttpResponse {
    let hash = path.into_inner();
    HttpResponse::Ok().body(format!("Get audiobook route: {}", hash))
}

pub async fn get_audiobook_download(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let hash = path.into_inner();
    HttpResponse::Ok().body(format!("Get audiobook download route: {}", hash))
}
