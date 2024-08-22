use crate::{
    inputs::auth::{AdminLogin, RegularLogin, RegularRegister},
    models::user::User,
    outputs::auth::AuthToken,
    state::AppState,
};
use anyhow::{anyhow, Result};
use rand::Rng;

pub async fn login(input: RegularLogin, state: &AppState) -> Result<AuthToken> {
    let db = &state.db;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash FROM users WHERE username = ?",
    )
    .bind(&input.username)
    .fetch_optional(db)
    .await?;

    let user = user.ok_or_else(|| anyhow!("Username does not exist"))?;

    let password_hash = format!("{:x}", md5::compute(&input.password));

    if password_hash != user.password_hash {
        return Err(anyhow!("Invalid password"));
    }

    let token = generate_token();
    let mut regular_tokens = state
        .regular_tokens
        .lock()
        .map_err(|_| anyhow!("Could not access regular_tokens"))?;
    regular_tokens.insert(token.clone(), input.username);

    Ok(AuthToken { token })
}

pub async fn admin_login(input: AdminLogin, state: &AppState) -> Result<AuthToken> {
    if input.password != state.admin_password {
        return Err(anyhow!("Invalid admin password"));
    }

    let token = generate_token();

    let mut admin_tokens = state
        .admin_tokens
        .lock()
        .map_err(|_| anyhow!("Could not access admin_tokens"))?;
    admin_tokens.push(token.clone());

    Ok(AuthToken { token })
}

pub async fn register(input: RegularRegister, state: &AppState) -> Result<AuthToken> {
    let db = &state.db;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash FROM users WHERE username = ?",
    )
    .bind(&input.username)
    .fetch_optional(db)
    .await?;

    if user.is_some() {
        return Err(anyhow!("Username already exists"));
    }

    let password_hash = format!("{:x}", md5::compute(&input.password));

    sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&input.username)
        .bind(&password_hash)
        .execute(db)
        .await?;

    let token = generate_token();
    let mut regular_tokens = state
        .regular_tokens
        .lock()
        .map_err(|_| anyhow!("Could not access regular_tokens"))?;
    regular_tokens.insert(token.clone(), input.username);

    Ok(AuthToken { token })
}

fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}
