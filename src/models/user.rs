use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}
