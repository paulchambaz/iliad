use actix_web::{web, HttpRequest, HttpResponse};

use crate::{
    inputs::position::InputPositionUpdate,
    services::position::{get_position_user_book, put_position_user_book},
    state::AppState,
};

pub async fn get_position(
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let hash = path.into_inner();

    let user = match req.headers().get("user").and_then(|h| h.to_str().ok()) {
        Some(user) => user,
        None => return HttpResponse::InternalServerError().body("Internal server error"),
    };

    match get_position_user_book(user.to_string(), hash, &state).await {
        Ok(position) => HttpResponse::Ok().json(position),
        Err(e) => {
            eprintln!("Login error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

pub async fn put_position(
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
    body: web::Json<InputPositionUpdate>,
) -> HttpResponse {
    let input = body.into_inner();
    let hash = path.into_inner();

    let user = match req.headers().get("user").and_then(|h| h.to_str().ok()) {
        Some(user) => user,
        None => return HttpResponse::InternalServerError().body("Internal server error"),
    };

    match put_position_user_book(user.to_string(), hash, input, &state).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(e) => {
            eprintln!("Login error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}
