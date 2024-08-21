use actix_web::{web, HttpResponse};

use crate::state::AppState;

pub async fn put_library_scan(state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().body("Put library scan route")
}

pub async fn put_library_cleanup(state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().body("Put library cleanup")
}
