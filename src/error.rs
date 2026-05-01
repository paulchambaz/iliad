use actix_web::HttpResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("conflict")]
    Conflict,
    #[error("{0}")]
    Internal(String),
    #[error("archive not ready")]
    ServiceUnavailable,
    #[error("{0}")]
    BadRequest(String),
}

impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::NotFound => HttpResponse::NotFound().body(self.to_string()),
            AppError::Unauthorized => HttpResponse::Unauthorized().body(self.to_string()),
            AppError::Conflict => HttpResponse::Conflict().body(self.to_string()),
            AppError::ServiceUnavailable => HttpResponse::ServiceUnavailable().body(self.to_string()),
            AppError::BadRequest(msg) => HttpResponse::BadRequest().body(msg.clone()),
            AppError::Internal(msg) => {
                tracing::error!("{}", msg);
                HttpResponse::InternalServerError().body("internal server error")
            }
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}
