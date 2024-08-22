use actix_web::{web, HttpResponse};

use crate::{
    services::library::{cleanup, scan_library},
    state::AppState,
};

pub async fn put_library_scan(state: web::Data<AppState>) -> HttpResponse {
    match scan_library(&state, true).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(e) => {
            eprintln!("Register error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

pub async fn put_library_cleanup(state: web::Data<AppState>) -> HttpResponse {
    match cleanup(&state).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(e) => {
            eprintln!("Register error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}
