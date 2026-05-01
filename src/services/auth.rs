use crate::{
    error::AppError,
    inputs::auth::{AdminLogin, RegularLogin, RegularRegister},
    outputs::auth::AuthToken,
    repo::user as user_repo,
    state::AppState,
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::distr::SampleString;
use rand_core::OsRng;
use std::time::Instant;

pub async fn login(input: RegularLogin, state: &AppState) -> Result<AuthToken, AppError> {
    let user = user_repo::find_by_username(&state.db, &input.username)
        .await?
        .ok_or(AppError::NotFound)?;

    let parsed = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::Internal("invalid password hash in db".into()))?;
    Argon2::default()
        .verify_password(input.password.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized)?;

    let token = generate_token();
    let mut tokens = state
        .regular_tokens
        .lock()
        .map_err(|_| AppError::Internal("lock poisoned".into()))?;
    tokens.retain(|_, (_, created)| created.elapsed() < state.token_ttl);
    tokens.insert(token.clone(), (input.username, Instant::now()));

    Ok(AuthToken { token })
}

pub async fn admin_login(input: AdminLogin, state: &AppState) -> Result<AuthToken, AppError> {
    if input.password != state.admin_password {
        return Err(AppError::Unauthorized);
    }

    let token = generate_token();
    let mut tokens = state
        .admin_tokens
        .lock()
        .map_err(|_| AppError::Internal("lock poisoned".into()))?;
    tokens.retain(|(_, created)| created.elapsed() < state.token_ttl);
    tokens.push((token.clone(), Instant::now()));

    Ok(AuthToken { token })
}

pub async fn register(input: RegularRegister, state: &AppState) -> Result<AuthToken, AppError> {
    if user_repo::find_by_username(&state.db, &input.username)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict);
    }

    let password_hash = hash_password(&input.password)?;
    user_repo::create(&state.db, &input.username, &password_hash).await?;

    let token = generate_token();
    let mut tokens = state
        .regular_tokens
        .lock()
        .map_err(|_| AppError::Internal("lock poisoned".into()))?;
    tokens.retain(|_, (_, created)| created.elapsed() < state.token_ttl);
    tokens.insert(token.clone(), (input.username, Instant::now()));

    Ok(AuthToken { token })
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AppError::Internal("password hashing failed".into()))
}

fn generate_token() -> String {
    rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 32)
}
