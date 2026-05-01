use actix_files::NamedFile;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::{web, HttpRequest, HttpResponse};

use crate::error::AppError;
use crate::services::audiobook::{get_audiobook_archive, get_audiobook_by_hash, list_audiobooks};
use crate::state::AppState;

pub async fn get_audiobooks(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let audiobooks = list_audiobooks(&state).await?;
    Ok(HttpResponse::Ok().json(audiobooks))
}

pub async fn get_audiobook(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let book = get_audiobook_by_hash(path.into_inner(), &state).await?;
    Ok(HttpResponse::Ok().json(book))
}

pub async fn get_audiobook_download(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let (filename, archive_path) = get_audiobook_archive(path.into_inner(), &state).await?;
    let file = NamedFile::open(&archive_path)?.set_content_disposition(ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::Filename(filename)],
    });
    Ok(file.into_response(&req))
}
