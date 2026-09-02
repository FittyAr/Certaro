//! Security ports for password hashing, token generation, and TOTP.

use crate::result::AppResult;

pub trait PasswordHasher: Send + Sync {
    fn hash_password(&self, password: &str) -> AppResult<String>;
    fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool>;
}

pub trait TokenPort: Send + Sync {
    fn generate_token(&self) -> String;
    fn hash_token(&self, token: &str) -> String;
}

pub trait TotpPort: Send + Sync {
    fn generate_secret(&self) -> String;
    fn verify_code(&self, secret: &str, code: &str) -> bool;
}
