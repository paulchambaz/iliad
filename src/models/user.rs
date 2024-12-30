use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct User {
    pub username: String, // primary key
    pub password_hash: String,
}
