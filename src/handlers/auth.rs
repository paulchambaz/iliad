use actix_web::{web, HttpResponse};

use crate::error::AppError;
use crate::inputs::auth::{AdminLogin, RegularLogin, RegularRegister};
use crate::services::auth::{admin_login, login, register};
use crate::state::AppState;

pub async fn post_auth_login(
    state: web::Data<AppState>,
    body: web::Json<RegularLogin>,
) -> Result<HttpResponse, AppError> {
    let token = login(body.into_inner(), &state).await?;
    Ok(HttpResponse::Ok().json(token))
}

pub async fn post_auth_admin(
    state: web::Data<AppState>,
    body: web::Json<AdminLogin>,
) -> Result<HttpResponse, AppError> {
    let token = admin_login(body.into_inner(), &state).await?;
    Ok(HttpResponse::Ok().json(token))
}

pub async fn post_auth_register(
    state: web::Data<AppState>,
    body: web::Json<RegularRegister>,
) -> Result<HttpResponse, AppError> {
    let token = register(body.into_inner(), &state).await?;
    Ok(HttpResponse::Ok().json(token))
}
