use actix_web::{web, HttpRequest, HttpResponse};

use crate::state::AppState;

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

    HttpResponse::Ok().body(format!("Get position route: {} to {}", hash, user))
}

pub async fn put_position(
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
    // body: web::Json<YourJsonType>
) -> HttpResponse {
    let hash = path.into_inner();

    let user = match req.headers().get("user").and_then(|h| h.to_str().ok()) {
        Some(user) => user,
        None => return HttpResponse::InternalServerError().body("Internal server error"),
    };

    HttpResponse::Ok().body(format!("Put audiobook route: {} to {}", hash, user))
}
