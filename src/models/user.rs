use sqlx::FromRow;

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct User {
    pub username: String, // primary key
    pub password_hash: String,
}
