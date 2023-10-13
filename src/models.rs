use chrono::NaiveDate;
use diesel::prelude::*;
use serde_derive::{Deserialize, Serialize};

use crate::schema::accounts;
use crate::schema::audiobooks;
use crate::schema::positions;

/// Represents an audiobook as stored in the database
#[derive(Queryable, Selectable)]
#[diesel(table_name = audiobooks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Audiobook {
    pub hash: String,
    pub title: String,
    pub author: String,
    pub date: String,
    pub description: String,
    pub genres: String,
    pub duration: i32,
    pub size: i32,
    pub path: String,
}

/// Reprensents the necessary fields to create a new audiobook record in the database
#[derive(Insertable)]
#[diesel(table_name = audiobooks)]
pub struct NewAudiobook<'a> {
    pub hash: &'a str,
    pub title: &'a str,
    pub author: &'a str,
    pub date: &'a str,
    pub description: &'a str,
    pub genres: &'a str,
    pub duration: i32,
    pub size: i32,
    pub path: &'a str,
}

/// Represents the format of an audiobook for the Json representation.
#[derive(Serialize)]
pub struct AudiobooksFmt {
    pub hash: String,
    pub title: String,
    pub author: String,
    pub date: String,
    pub description: String,
    pub genres: String,
    pub duration: i32,
    pub size: i32,
}

/// Represents an audiobook for scanning purposes.
pub struct AudiobookScan {
    pub hash: String,
    pub title: String,
    pub author: String,
    pub date: String,
    pub description: String,
    pub genres: String,
    pub duration: i32,
    pub size: i32,
    pub path: String,
}

/// Represents a user's position within the audiobook in the database
#[derive(Queryable, Selectable)]
#[diesel(table_name = positions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Position {
    pub hash: String,
    pub username: String,
    pub file: String,
    pub position: i32,
    pub last_modified: NaiveDate,
}

/// Reprensents the necessary fields to create a new position record in the database
#[derive(Insertable)]
#[diesel(table_name = positions)]
pub struct NewPosition<'a> {
    pub hash: &'a str,
    pub username: &'a str,
    pub file: &'a str,
    pub position: i32,
    pub last_modified: chrono::NaiveDate,
}

/// Reprensents the necessary fields to update a new position record in the database
#[derive(AsChangeset)]
#[diesel(table_name = positions)]
pub struct UpdatePosition<'a> {
    pub hash: &'a str,
    pub username: &'a str,
    pub file: &'a str,
    pub position: i32,
    pub last_modified: chrono::NaiveDate,
}

/// Represents the format of an audiobook for the Json representation
#[derive(Serialize, Deserialize)]
pub struct PositionFmt {
    pub file: String,
    pub position: i32,
}

/// Represents a user's account information as stored in the database
#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = accounts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Account {
    pub username: String,
    pub password: String,
    pub key: String,
}

/// Reprensents the necessary fields to create a new account record in the database
#[derive(Insertable)]
#[diesel(table_name = accounts)]
pub struct NewAccount<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub key: &'a str,
}

/// Represents user connection information for authentication purposes, for the Json representation
#[derive(Deserialize)]
pub struct Connection {
    pub username: String,
    pub password: String,
}

/// Represents an API key associated with a user account, for the Json representation
#[derive(Serialize)]
pub struct ApiKey {
    pub key: String,
}
