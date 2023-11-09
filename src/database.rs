use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use rand::Rng;
use std::env;

use crate::errors::IliadError;
use crate::models::{
    Account, ApiKey, Audiobook, AudiobookScan, AudiobooksFmt, NewAccount, NewAudiobook,
    NewPosition, Position, PositionFmt, UpdatePosition,
};
use crate::schema::{accounts, audiobooks, positions};

/// Establishes a connection for the database
///
/// This function uses the "ILIAD_DB_PATH" environment variable to determine the database URL.
///
/// # Returns
///
/// On success, a connected `SqliteConnection` instance. On failure, the relevant error case if the
/// database is not available or the connection could not be established.
pub fn establish_connection() -> Result<SqliteConnection, IliadError> {
    dotenv().ok();

    let database_url = match env::var("ILIAD_DB_PATH") {
        Ok(url) => url,
        Err(_) => return Err(IliadError::DatabaseUnavailable),
    };

    match SqliteConnection::establish(&database_url) {
        Ok(conn) => Ok(conn),
        Err(_) => Err(IliadError::DatabaseUnavailable),
    }
}

/// Authenticates a user using a key from the database
///
/// # Arguments
///
/// * `conn` - The connection to the sqlite database
/// * `key`  - The key used for authentication, used to retrieve the account
///
/// # Returns
///
/// On success, the username associated with the key. On failure, the relevant error case, if the
/// key is not present in the database.
pub fn auth(conn: &mut SqliteConnection, key: String) -> Result<String, IliadError> {
    match accounts::dsl::accounts
        .filter(accounts::dsl::key.eq(&key))
        .first::<Account>(conn)
    {
        Ok(account) => Ok(account.username),
        Err(Error::NotFound) => Err(IliadError::DatabaseNotFound),
        Err(_) => Err(IliadError::DatabaseUnavailable),
    }
}

/// Updates the audiobooks in the database.
///
/// Clears the existing audiobooks, then inserts the provided audiobooks into the database
/// atomically.
///
/// # Arguments
///
/// * `conn`       - The connection to the sqlite database
/// * `audiobooks` - A vector of `AudiobookScan` to be updated in the database
///
/// # Returns
///
/// On success, ok. On failure, a error case indicating that the database operation has failed.
pub fn update_audiobooks(
    conn: &mut SqliteConnection,
    audiobooks: Vec<AudiobookScan>,
) -> Result<(), IliadError> {
    conn.transaction::<_, Error, _>(|connection| {
        diesel::delete(audiobooks::table).execute(connection)?;
        let mut new_audiobooks: Vec<NewAudiobook> = Vec::new();

        for audiobook in &audiobooks {
            let new_audiobook = NewAudiobook {
                hash: &audiobook.hash,
                title: &audiobook.title,
                author: &audiobook.author,
                date: &audiobook.date,
                description: &audiobook.description,
                genres: &audiobook.genres,
                duration: audiobook.duration,
                size: audiobook.size,
                path: &audiobook.path,
            };
            new_audiobooks.push(new_audiobook);
        }

        diesel::insert_into(audiobooks::table)
            .values(&new_audiobooks)
            .execute(connection)?;
        Ok(())
    })
    .map_err(|_| IliadError::DatabaseUnavailable)?;

    Ok(())
}

/// Fetches all audiobook from the database
///
/// # Arguements
///
/// * `conn` - The connection to the sqlite database
///
/// # Returns
///
/// On success, a vector of `AudiobookFmt`, containing the fetched audiobooks. On failure, the
/// relevant error case.
pub fn get_audiobooks(conn: &mut SqliteConnection) -> Result<Vec<AudiobooksFmt>, IliadError> {
    match audiobooks::dsl::audiobooks
        .select(Audiobook::as_select())
        .load(conn)
    {
        Ok(audiobooks) => Ok(audiobooks
            .into_iter()
            .map(|book| AudiobooksFmt {
                hash: book.hash,
                title: book.title,
                author: book.author,
                date: book.date,
                description: book.description,
                genres: book.genres,
                duration: book.duration,
                size: book.size,
            })
            .collect()),
        Err(_) => Err(IliadError::DatabaseUnavailable),
    }
}

/// Fetches a specific audiobook from the database using its hash.
///
/// # Arguments
///
/// * `conn` - The connection to the sqlite database
/// * `hash` - The unique hash of the audiobook to be retrieved
///
/// # Returns
///
/// On success, the `Audiobook` corresponding to the given hash. On failure, the relevant error
/// case, if the audiobook is not found in the database.
pub fn get_audiobook(conn: &mut SqliteConnection, hash: String) -> Result<Audiobook, IliadError> {
    match audiobooks::dsl::audiobooks
        .filter(audiobooks::dsl::hash.eq(&hash))
        .first::<Audiobook>(conn)
    {
        Ok(audiobook) => Ok(audiobook),
        Err(Error::NotFound) => Err(IliadError::DatabaseNotFound),
        Err(_) => Err(IliadError::DatabaseUnavailable),
    }
}

/// Retries the position associated with a specific hash and username from the database.
///
/// # Arguements
///
/// * `conn`     - The connection to the sqlite database
/// * `hash`     - A unique hash used to identify the position
/// * `username` - The username used to identify the position
///
/// # Returns
///
/// On success, the position associated with the hash and username, as `PositionFmt`. On failure,
/// the relevant error case.
pub fn get_position(
    conn: &mut SqliteConnection,
    hash: String,
    username: String,
) -> Result<PositionFmt, IliadError> {
    match positions::dsl::positions
        .filter(
            positions::dsl::hash
                .eq(&hash)
                .and(positions::dsl::username.eq(&username)),
        )
        .first::<Position>(conn)
    {
        Ok(position) => Ok(PositionFmt {
            file: position.file,
            position: position.position,
        }),
        Err(Error::NotFound) => Err(IliadError::DatabaseNotFound),
        Err(_) => Err(IliadError::DatabaseUnavailable),
    }
}

/// Updates or inserts a position for a specific hash and username in the database.
///
/// If a position associated with the given hash and username exists, it is updated. If it does not
/// exist, a new position is inserted into the database.
///
/// # Arguements
///
/// * `conn`     - The connection to the sqlite database
/// * `hash`     - A unique hash used to identify the position
/// * `username` - The username used to identify the position
/// * `position` - The new position data to be updated or inserted
pub fn put_position(
    conn: &mut SqliteConnection,
    hash: String,
    username: String,
    position: PositionFmt,
) -> Result<IliadError, IliadError> {
    let exists = match positions::dsl::positions
        .filter(
            positions::dsl::hash
                .eq(&hash)
                .and(positions::dsl::username.eq(&username)),
        )
        .first::<Position>(conn)
    {
        Ok(_) => true,
        Err(Error::NotFound) => false,
        Err(_) => return Err(IliadError::DatabaseUnavailable),
    };

    let now = Utc::now().naive_utc();

    if exists {
        let update_position = UpdatePosition {
            hash: &hash,
            username: &username,
            file: &position.file,
            position: position.position,
            last_modified: now.into(),
        };

        match diesel::update(
            positions::dsl::positions
                .filter(positions::dsl::hash.eq(&hash))
                .filter(positions::dsl::username.eq(&username)),
        )
        .set(&update_position)
        .execute(conn)
        {
            Ok(_) => Ok(IliadError::Ok),
            Err(Error::NotFound) => Err(IliadError::DatabaseNotFound),
            Err(_) => Err(IliadError::DatabaseUnavailable),
        }
    } else {
        let new_position = NewPosition {
            hash: &hash,
            username: &username,
            file: &position.file,
            position: position.position,
            last_modified: now.into(),
        };

        match diesel::insert_into(positions::table)
            .values(&new_position)
            .execute(conn)
        {
            Ok(_) => Ok(IliadError::Ok),
            Err(_) => Err(IliadError::DatabaseUnavailable),
        }
    }
}

pub fn login(
    conn: &mut SqliteConnection,
    username: String,
    password: String,
) -> Result<ApiKey, IliadError> {
    let account = match accounts::dsl::accounts
        .filter(accounts::dsl::username.eq(&username))
        .first::<Account>(conn)
    {
        Ok(account) => account,
        Err(Error::NotFound) => return Err(IliadError::DatabaseNotFound),
        Err(_) => return Err(IliadError::DatabaseUnavailable),
    };

    if account.password != password {
        return Err(IliadError::InvalidClientConnection);
    }

    Ok(ApiKey { key: account.key })
}

pub fn register(
    conn: &mut SqliteConnection,
    username: String,
    password: String,
) -> Result<ApiKey, IliadError> {
    match accounts::dsl::accounts
        .filter(accounts::dsl::username.eq(&username))
        .first::<Account>(conn)
    {
        Ok(_) => return Err(IliadError::DatabaseAlreadyFound),
        Err(Error::NotFound) => {}
        Err(_) => return Err(IliadError::DatabaseUnavailable),
    };

    let keys = match accounts::dsl::accounts
        .select(accounts::dsl::key)
        .load::<String>(conn)
    {
            Ok(keys) => keys,
            Err(_) => return Err(IliadError::DatabaseUnavailable),
    };

    let mut key = String::with_capacity(32);
    let mut rng = rand::thread_rng();

    loop {
        for _ in 0..16 {
            key.push_str(&format!("{:02x}", rng.gen::<u8>()));
        }

        if !keys.contains(&key) {
            break;
        }
    }

    let new_account = NewAccount {
        username: &username,
        password: &password,
        key: &key,
    };

    if diesel::insert_into(accounts::table)
        .values(new_account)
        .execute(conn).is_err() {
        return Err(IliadError::DatabaseUnavailable);
    };
    
    Ok(ApiKey { key })
}

pub fn cleanup_positions() -> Result<(), ()> {
    dotenv().ok();

    let cleanup_interval_str: String = match env::var("ILIAD_CLEANUP_INTERVAL") {
        Ok(value) if !value.is_empty() => value,
        _ => return Err(()),
    };

    let cleanup_interval: i64 = cleanup_interval_str.parse().map_err(|_| ())?;

    let conn = &mut match establish_connection() {
        Ok(conn) => conn,
        Err(_) => return Err(()),
    };

    let cutoff = Utc::now().naive_utc().date() - Duration::days(cleanup_interval);

    match diesel::delete(positions::table.filter(positions::last_modified.lt(cutoff)))
        .execute(conn)
        .map_err(|_| ())
    {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}


pub fn encrypt(data: &str, password: &str) -> String {
    // use AEAD algorithm and crate
    // use pbkdf2 for the creating the password hash
}
