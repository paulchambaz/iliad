use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};

use crate::error::AppError;
use crate::inputs::position::InputPositionUpdate;
use crate::services::position::{get_position_user_book, put_position_user_book};
use crate::state::AppState;

pub async fn get_position(
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user = req
        .extensions()
        .get::<String>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let position = get_position_user_book(user, path.into_inner(), &state).await?;
    Ok(HttpResponse::Ok().json(position))
}

pub async fn put_position(
    state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
    body: web::Json<InputPositionUpdate>,
) -> Result<HttpResponse, AppError> {
    let user = req
        .extensions()
        .get::<String>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    put_position_user_book(user, path.into_inner(), body.into_inner(), &state).await?;
    Ok(HttpResponse::Ok().finish())
}
