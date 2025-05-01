use argon2::{
    Algorithm, Params, Version, Argon2, PasswordHasher, PasswordVerifier, 
    password_hash::{
        rand_core::OsRng,
        PasswordHash, SaltString
    },
};
use crate::models::Argon;
use log::error;

impl Argon {
    pub async fn verify_hash(hash: &str, value: &str) -> Result<(), argon2::password_hash::Error> {
        let argon2 = Argon2::default();
        let parsed_hash = match PasswordHash::new(value) {
            Ok(hash) => hash,
            Err(e) => {
                error!("Ошибка парсинга в хеш: {e}");
                return Err(e)
            }
        };

        match argon2.verify_password(hash.as_bytes(), &parsed_hash) {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!("Ошибка проверки хеша: {e}");
                return Err(e);
            }
        }
    }
}
