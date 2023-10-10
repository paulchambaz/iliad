use rocket::config::{Config, Environment};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use rocket::{get, post, put};
use rocket::routes;
use rocket_contrib::json::Json;
// use rocket_contrib::json::{Json, JsonValue};
// use serde_json::json;
use std::net::IpAddr;

use crate::controllers;
use crate::models;

/// Creates and configures a Rocket instance with specific routes
///
/// This function sets up a Rocket web server instance with various routes for handling the
/// audiobook related operations and user authentification. The sets of routes can be expanded
/// based on the `register` flag.
///
/// # Arguments
///
/// * `address`  - The Ip Adress the server will be bind to
/// * `port`     - The port the server will listen on
/// * `register` - Flag to indicate if the registration route should be included
///
/// # Returns
///
/// A configured instance of a Rocket web server, ready to be launched
pub fn create_rocket(address: IpAddr, port: u16, register: bool) -> rocket::Rocket {
    let config = Config::build(Environment::Development)
        .address(address.to_string())
        .port(port)
        .unwrap();

    rocket::custom(config)
        .mount(
            "/",
            if register {
                routes![
                    get_audiobooks_route,
                    get_audiobook_route,
                    get_audiobook_position_route,
                    put_audiobook_position_route,
                    post_login_route,
                    post_register_route,
                ]
            } else {
                routes![
                    get_audiobooks_route,
                    get_audiobook_route,
                    get_audiobook_position_route,
                    put_audiobook_position_route,
                    post_login_route,
                ]
            },
        )
}

/// Rocket route to retrieve the list of available audiobooks. Serves as a connector between the
/// rocket server and the controller.
///
/// # Arguments
///
/// * `_token` - The Auth token sent for authentification
///
/// # Returns
///
/// On success, a Json containing the list of audiobooks. On failure the HTTP error status code
#[get("/audiobooks")]
fn get_audiobooks_route(_token: AuthToken) -> Result<Json<Vec<models::AudiobooksFmt>>, Status> {
    controllers::get_audiobooks()
}

/// Rocket route to retrieve the compressed archive binary data of a single audiobook Serves as a
/// connector between the rocket server and the controller.
///
/// # Arguments
///
/// * `hash`   - The hash of the audiobook requested
/// * `_token` - The Auth token sent for authentification
///
/// # Returns
///
/// On success, the binary data of the compressed archive of the audiobook. On failure the HTTP error
/// status code
#[get("/audiobook/<hash>")]
fn get_audiobook_route(hash: String, _token: AuthToken) -> Result<Vec<u8>, Status> {
    controllers::get_audiobook(hash)
}

/// Rocket route to retrieve the position data of a specific audiobook for the authenticated
/// user. Serves as a connector between the rocket server and the controller.
///
/// # Arguments
///
/// * `hash`  - The hash of the audiobook for the position requested
/// * `token` - The Auth token sent for authentification
///
/// # Returns
///
/// On success, a Json containing the position information for the audiobook for the authenticated
/// user. On failure, the HTTP error status code
#[get("/audiobook/<hash>/position")]
fn get_audiobook_position_route(
    hash: String,
    token: AuthToken,
) -> Result<Json<models::PositionFmt>, Status> {
    let AuthToken(username) = token;
    controllers::get_audiobook_position(hash, username)
}

/// Rocket route to update the position data of a specific audiobook for the authenticated user.
/// Serves as a connector between the rocket server and the controller.
///
/// # Arguments
///
/// * `hash`     - The hash of the audiobook for the position requested
/// * `position` - The Json of the position information used to update
/// * `token`    - The Auth token sent for authentification
///
/// # Returns
///
/// # On success, a success HTTP code. On failure, the HTTP error status code.
#[put("/audiobook/<hash>/position", data = "<position>")]
fn put_audiobook_position_route(
    hash: String,
    position: Json<models::PositionFmt>,
    token: AuthToken,
) -> Result<Status, Status> {
    let AuthToken(username) = token;
    controllers::put_audiobook_position(hash, username, position)
}

/// Rocker route to login to a given account and receive the api key.
///
/// # Arguments
///
/// * `account` - A Json containing username and password
///
/// # Returns
///
/// On success, a Json containing the api key for the authenticated user. On failure, the HTTP
/// error status code
#[post("/login", data = "<account>")]
fn post_login_route(account: Json<models::Connection>) -> Result<Json<models::ApiKey>, Status> {
    controllers::post_login(account)
}

/// Rocker route to register a given account with a password and receive the api key.
///
/// # Arguments
///
/// * `account` - A Json containing username and password
///
/// # Returns
///
/// On success, a Json containing the api key for the newly authenticated user. On failure, the
/// HTTP error status code
#[post("/register", data = "<account>")]
fn post_register_route(account: Json<models::Connection>) -> Result<Json<models::ApiKey>, Status> {
    controllers::post_register(account)
}

/// Reprensent the authetication token for a user
///
/// If the authentication is successful, an `AuthToken` containing the username is returned.
/// Otherwise, appropriate error responses (`BadRequest` or `Unauthrized`) are returned.
pub struct AuthToken(pub String);
impl<'a, 'r> FromRequest<'a, 'r> for AuthToken {
    type Error = ();

    /// Attempts to extract the AuthToken from the provided request's header.
    ///
    /// If a given "Auth" token is sent, the header's value is checked again the authentication
    /// system
    fn from_request(request: &'a Request<'r>) -> request::Outcome<AuthToken, Self::Error> {
        let keys: Vec<_> = request.headers().get("Auth").collect();
        if keys.len() != 1 {
            return Outcome::Failure((Status::BadRequest, ()));
        }

        let key = keys[0].to_string();

        if key.is_empty() {
            return Outcome::Failure((Status::BadRequest, ()));
        }

        match controllers::get_auth(keys[0].to_string()) {
            Ok(username) => Outcome::Success(AuthToken(username)),
            Err(_) => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}
