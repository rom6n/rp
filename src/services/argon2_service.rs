use argon2::{
    Algorithm, Params, Version, Argon2, PasswordHasher, PasswordVerifier, 
    password_hash::{
        rand_core::OsRng,
        PasswordHash, SaltString
    },
};
use tokio::task::spawn_blocking;

use crate::models::{Argon, ArgonError};
use log::error;


impl Argon {
    pub async fn verify_hash(hash: &str, value: &str) -> Result<(), ArgonError> {
        let hash = hash.to_owned();
        let value = value.to_owned();

        let process = tokio::task::spawn_blocking(move || {
            let argon2 = Argon2::default();
            let parsed_hash = match PasswordHash::new(&hash) {
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
        });
        match process.await {
            Ok(Ok(())) => return Ok(()),
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                error!("Ошибка запуска verify spawn_blocking: {e}");
                return Err(ArgonError::TokioError)
            }
        }
    }

    pub async fn hash_str(data: &str) -> Result<String, ArgonError> {
        let data = data.to_owned();

        let hashing = spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);

            let params = match Params::new(64 * 1024, 2, 1, Some(32)) {
                Ok(val) => val,
                Err(e) => {
                    error!("Ошибка создания params в argon2: {e}");
                    return Err(ArgonError::ParamsError);
                }
            };

            let salt_string = salt.to_string();
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            match argon2.hash_password(data.as_bytes(), &SaltString::from_b64(&salt_string).expect("Передана неверная salt строка")) {
                Ok(hash) => return Ok(hash.to_string()),
                Err(e) => {
                    error!("Ошибка хеширования: {e}");
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
            Err(e) => {
                error!("Ошибка запуска hash spawn_blocking: {e}");
                return Err(ArgonError::TokioError)
            }
        }
    }


}
