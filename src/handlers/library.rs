use actix_web::{web, HttpResponse};

use crate::error::AppError;
use crate::services::library::{cleanup, scan_library};
use crate::state::AppState;

pub async fn put_library_scan(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    scan_library(&state).await?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn put_library_cleanup(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    cleanup(&state).await?;
    Ok(HttpResponse::Ok().finish())
}
