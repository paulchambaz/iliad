use rocket::http::Status;
use rocket_contrib::json::Json;

use crate::database;
use crate::errors;
use crate::models;
use crate::scan;

use errors::IliadError;

/// Retries the username associated with a given authentication key, if a valid account exists for
/// the key in the application's database. Returns the valid HTTP response, with proper status, to
/// be sent to a given route.
///
/// # Arguments
///
/// * `key` - A string representing the authentication key used to fetch the associated username
///
/// # Rerturns
///
/// On success, the username associated with the given key. On failure, the relevant HTTP status
/// indicating the nature of the error.
pub fn get_auth(key: String) -> Result<String, Status> {
    let conn = &mut match database::establish_connection() {
        Ok(conn) => conn,
        Err(_) => return Err(Status::ServiceUnavailable),
    };

    match database::auth(conn, key) {
        Ok(username) => return Ok(username),
        Err(IliadError::DatabaseNotFound) => return Err(Status::Unauthorized),
        Err(IliadError::DatabaseUnavailable) => return Err(Status::ServiceUnavailable),
        Err(_) => return Err(Status::InternalServerError),
    }
}

/// Returns a list of available audiobooks. Returns the valid HTTP response, with proper status, to
/// be sent to a given route.
///
/// # Returns
///
/// On success, a Json containing the list of audiobooks. On failure the HTTP error status code
pub fn get_audiobooks() -> Result<Json<Vec<models::AudiobooksFmt>>, Status> {
    let conn = &mut match database::establish_connection() {
        Ok(conn) => conn,
        Err(_) => return Err(Status::ServiceUnavailable),
    };

    match database::get_audiobooks(conn) {
        Ok(audiobooks) => return Ok(Json(audiobooks)),
        Err(IliadError::DatabaseUnavailable) => return Err(Status::ServiceUnavailable),
        Err(_) => return Err(Status::InternalServerError),
    }
}

/// Returns binary data of the compressed archive for a given audiobook. Returns a valid HTTP
/// response, with proper status, to be sent to a given route.
///
/// # Arguments
///
/// * `hash`   - The hash of the audiobook requested
/// # Returns
///
/// On success, the binary data of the compressed archive of the audiobook. On failure the HTTP error
/// status code
pub fn get_audiobook(hash: String) -> Result<Vec<u8>, Status> {
    let conn = &mut match database::establish_connection() {
        Ok(conn) => conn,
        Err(_) => return Err(Status::ServiceUnavailable),
    };

    let path = match database::get_audiobook(conn, hash) {
        Ok(audiobook) => audiobook.path.into(),
        Err(IliadError::DatabaseNotFound) => return Err(Status::NotFound),
        Err(IliadError::DatabaseUnavailable) => return Err(Status::ServiceUnavailable),
        Err(_) => return Err(Status::InternalServerError),
    };

    match scan::archive_directory(&path) {
        Ok(binary_data) => Ok(binary_data),
        Err(_) => return Err(Status::InternalServerError),
    }
}

/// Returns the position information for an audiobook and a user. Returns a valid HTTP response,
/// with proper status, to be sent to a given route.
///
/// # Arguments
///
/// * `hash`  - The hash of the audiobook for the position requested
/// * `username` - The username for the position requested
///
/// # Returns
///
/// On success, a Json containing the position information for the audiobook for the authenticated
/// user. On failure, the HTTP error status code
pub fn get_audiobook_position(
    hash: String,
    username: String,
) -> Result<Json<models::PositionFmt>, Status> {
    let conn = &mut match database::establish_connection() {
        Ok(conn) => conn,
        Err(_) => return Err(Status::ServiceUnavailable),
    };

    match database::get_position(conn, hash, username) {
        Ok(position) => Ok(Json(position)),
        Err(IliadError::DatabaseUnavailable) => return Err(Status::ServiceUnavailable),
        Err(IliadError::DatabaseNotFound) => return Err(Status::NotFound),
        Err(_) => return Err(Status::InternalServerError),
    }
}

/// Update the position information for an audiobook and a user. Returns a valid HTTP response,
/// with proper status, to be sent to a given route.
///
/// # Arguments
///
/// * `hash`     - The hash of the audiobook for the position requested
/// * `position` - The Json of the position information used to update
/// * `username`    - The username for the position requested
///
/// # Returns
///
/// # On success, a success HTTP code. On failure, the HTTP error status code.
pub fn put_audiobook_position(
    hash: String,
    username: String,
    position: Json<models::PositionFmt>,
) -> Result<Status, Status> {
    let conn = &mut match database::establish_connection() {
        Ok(conn) => conn,
        Err(_) => return Err(Status::ServiceUnavailable),
    };

    let Json(position) = position;

    match database::put_position(conn, hash, username, position) {
        Ok(_) => return Ok(Status::Ok),
        Err(_) => return Err(Status::InternalServerError),
    }
}

/// Initiates the login process for a user. Returns a valid HTTP response, with proper status, to
/// be sent to a given route.
///
/// # Arguments
///
/// * `account` - A Json containing username and password
///
/// # Returns
///
/// On success, a Json containing the api key for the newly authenticated user. On failure, the
/// HTTP error status code
pub fn post_login(account: Json<models::Connection>) -> Result<Json<models::ApiKey>, Status> {
    let conn = &mut match database::establish_connection() {
        Ok(conn) => conn,
        Err(_) => return Err(Status::ServiceUnavailable),
    };

    let Json(account) = account;

    match database::login(conn, account.username, account.password) {
        Ok(api_key) => return Ok(Json(api_key)),
        Err(IliadError::DatabaseUnavailable) => return Err(Status::ServiceUnavailable),
        Err(IliadError::DatabaseNotFound) => return Err(Status::Unauthorized),
        Err(_) => return Err(Status::InternalServerError),
    }
}

/// Initiates the registration process for a new user. Returns a valid HTTP response, with proper
/// status, to be sent to a given route.
///
/// # Arguments
///
/// * `account` - A Json containing username and password
///
/// # Returns
///
/// On success, a Json containing the api key for the newly authenticated user. On failure, the
/// HTTP error status code
pub fn post_register(account: Json<models::Connection>) -> Result<Json<models::ApiKey>, Status> {
    let conn = &mut match database::establish_connection() {
        Ok(conn) => conn,
        Err(_) => return Err(Status::ServiceUnavailable),
    };

    let Json(account) = account;

    match database::register(conn, account.username, account.password) {
        Ok(api_key) => return Ok(Json(api_key)),
        Err(IliadError::DatabaseUnavailable) => return Err(Status::ServiceUnavailable),
        Err(IliadError::DatabaseAlreadyFound) => return Err(Status::Unauthorized),
        Err(_) => return Err(Status::InternalServerError),
    }
}
