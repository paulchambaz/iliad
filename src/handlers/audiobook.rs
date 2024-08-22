use actix_web::{http::header, web, HttpResponse};

use crate::{
    services::audiobook::{get_audiobook_archive, get_audiobook_by_hash, list_audiobooks},
    state::AppState,
};

pub async fn get_audiobooks(state: web::Data<AppState>) -> HttpResponse {
    match list_audiobooks(&state).await {
        Ok(audiobooks) => HttpResponse::Ok().json(audiobooks),
        Err(e) => {
            eprintln!("Get audiobooks error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

pub async fn get_audiobook(state: web::Data<AppState>, path: web::Path<String>) -> HttpResponse {
    let hash = path.into_inner();
    match get_audiobook_by_hash(hash, &state).await {
        Ok(audiobooks) => HttpResponse::Ok().json(audiobooks),
        Err(e) => {
            eprintln!("Get audiobooks error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

pub async fn get_audiobook_download(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let hash = path.into_inner();
    match get_audiobook_archive(hash, &state).await {
        Ok((filename, bytes)) => HttpResponse::Ok()
            .insert_header(header::ContentType::octet_stream())
            .insert_header(header::ContentDisposition::attachment(filename))
            .body(bytes),
        Err(e) => {
            eprintln!("Error getting audiobook archive: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}
