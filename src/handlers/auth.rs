use actix_web::{web, HttpResponse};

use crate::inputs::auth::{AdminLogin, RegularLogin, RegularRegister};
use crate::services::auth::{admin_login, login, register};
use crate::state::AppState;

pub async fn post_auth_login(
    state: web::Data<AppState>,
    body: web::Json<RegularLogin>,
) -> HttpResponse {
    let input = body.into_inner();
    match login(input, &state).await {
        Ok(token) => HttpResponse::Ok().json(token),
        Err(e) => {
            eprintln!("Login error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

pub async fn post_auth_admin(
    state: web::Data<AppState>,
    body: web::Json<AdminLogin>,
) -> HttpResponse {
    let input = body.into_inner();
    match admin_login(input, &state).await {
        Ok(token) => HttpResponse::Ok().json(token),
        Err(e) => {
            eprintln!("Admin login error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

pub async fn post_auth_register(
    state: web::Data<AppState>,
    body: web::Json<RegularRegister>,
) -> HttpResponse {
    let input = body.into_inner();
    match register(input, &state).await {
        Ok(token) => HttpResponse::Ok().json(token),
        Err(e) => {
            eprintln!("Register error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}
