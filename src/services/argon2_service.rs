use argon2::{
    Algorithm, Params, Version, Argon2, PasswordHasher, PasswordVerifier, 
    password_hash::{
        rand_core::OsRng,
        PasswordHash, SaltString
    },
};
use thiserror::Error;

use crate::models::{Argon, ArgonError};
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

    pub async fn hash_str(data: &str) -> Result<String, ArgonError> {
        let salt = SaltString::generate(&mut OsRng);

        let mut params = match Params::new(64440, 3, 2, None) {
            Ok(val) => val,
            Err(e) => {
                error!("Ошибка создания params в argon2: {e}");
                return Err(ArgonError::ParamsError);
            }
        };

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        match argon2.hash_password(data.as_bytes(), &salt) {
            Ok(hash) => return Ok(hash.to_string()),
            Err(e) => {
                error!("Не удалось хешировать данные: {e}");
                return Err(ArgonError::HashError)
            }
        }
    }
}
