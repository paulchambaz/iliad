use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RegularLogin {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct AdminLogin {
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct RegularRegister {
    pub username: String,
    pub password: String,
}
