use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct AuthToken {
    pub token: String,
}
