use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http::header::{HeaderName, HeaderValue},
    middleware::Next,
    web::Data,
    Error,
};

use crate::state::AppState;

enum UserType {
    Admin,
    Regular(String),
}

enum AuthError {
    Unauthorized,
    InternalError(String),
}

pub async fn standard_auth(
    mut req: ServiceRequest,
    next: Next<impl actix_web::body::MessageBody>,
) -> Result<ServiceResponse<impl actix_web::body::MessageBody>, Error> {
    let token = extract_token(&req)?;

    match validate_token(&req, token) {
        Ok(UserType::Admin) => {
            insert_user_header(&mut req, "admin");
            next.call(req).await
        }
        Ok(UserType::Regular(username)) => {
            insert_user_header(&mut req, &username);
            next.call(req).await
        }
        Err(AuthError::Unauthorized) => Err(actix_web::error::ErrorUnauthorized("Invalid token")),
        Err(AuthError::InternalError(e)) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

pub async fn admin_auth(
    mut req: ServiceRequest,
    next: Next<impl actix_web::body::MessageBody>,
) -> Result<ServiceResponse<impl actix_web::body::MessageBody>, Error> {
    let token = extract_token(&req)?;

    match validate_token(&req, token) {
        Ok(UserType::Admin) => {
            insert_user_header(&mut req, "admin");
            next.call(req).await
        }
        Ok(UserType::Regular(_)) | Err(AuthError::Unauthorized) => {
            Err(actix_web::error::ErrorUnauthorized("Invalid token"))
        }
        Err(AuthError::InternalError(e)) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

fn extract_token(req: &ServiceRequest) -> Result<String, Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Missing auth token"))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid auth header"))?;

    auth_str
        .strip_prefix("Bearer ")
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Invalid auth format"))
        .map(|s| s.to_string())
}

fn validate_token(req: &ServiceRequest, token: String) -> Result<UserType, AuthError> {
    let state = req
        .app_data::<Data<AppState>>()
        .ok_or(AuthError::InternalError("AppState not found".to_string()))?;

    // Check admin tokens
    let admin_tokens = state
        .admin_tokens
        .lock()
        .map_err(|_| AuthError::InternalError("Failed to acquire admin tokens lock".to_string()))?;

    if admin_tokens.contains(&token) {
        return Ok(UserType::Admin);
    }

    // Check regular tokens
    let regular_tokens = state.regular_tokens.lock().map_err(|_| {
        AuthError::InternalError("Failed to acquire regular tokens lock".to_string())
    })?;

    if let Some(username) = regular_tokens.get(&token) {
        Ok(UserType::Regular(username.clone()))
    } else {
        Err(AuthError::Unauthorized)
    }
}

fn insert_user_header(req: &mut ServiceRequest, username: &str) {
    req.headers_mut().insert(
        HeaderName::from_static("user"),
        HeaderValue::from_str(username).unwrap(),
    );
}
