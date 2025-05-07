use argon2::{
    Algorithm, Params, Version, Argon2, PasswordHasher, PasswordVerifier, 
    password_hash::{
        rand_core::OsRng,
        PasswordHash, SaltString
    },
};

use tokio::task::spawn_blocking;
use std::sync::Arc;

use crate::models::{Argon, ArgonError};
use log::error;


impl Argon {
    pub async fn verify_hash(hash: &str, value: &str) -> Result<(), ArgonError> {
        let argon2 = Argon2::default();
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(hash) => hash,
            Err(e) => {
                error!("Ошибка парсинга в хеш: {e}");
                return Err(ArgonError::ParseError)
            }
        };

        match argon2.verify_password(value.as_bytes(), &parsed_hash) {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!("Ошибка проверки хеша: {e}");
                return Err(ArgonError::VerifyHashError);
            }
        }
    }

    pub async fn hash_str(data: &str) -> Result<String, ArgonError> {
        let salt = SaltString::generate(&mut OsRng);

        let params = match Params::new(64 * 1024, 2, 1, Some(32)) {
            Ok(val) => val,
            Err(e) => {
                error!("Ошибка создания params в argon2: {e}");
                return Err(ArgonError::ParamsError);
            }
        };
        let data = data.to_owned();
        let salt = salt.to_string();
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let hashing = spawn_blocking(move || {
            match argon2.hash_password(data.as_bytes(), &SaltString::from_b64(&salt).expect("Передана неверная salt строка")) {
                Ok(hash) => return Ok(hash.to_string()),
                Err(_) => {
                    return Err(ArgonError::HashError)
                }
            }
        });

        match hashing.await {
            Ok(Ok(hash)) => return Ok(hash),
            Ok(Err(e)) => {
                error!("Не удалось хешировать данные: {e}");
                return Err(ArgonError::HashError)
            },
            Err(_) => return Err(ArgonError::TokioError)
        }
    }


}
