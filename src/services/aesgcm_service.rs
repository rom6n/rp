use aes_gcm::{aead::{generic_array::GenericArray, Aead, AeadCore, AeadMut, KeyInit}, Aes256Gcm, Key, Nonce};
use dotenv::dotenv;
use std::env;
use tokio::fs;
use std::io::{self, Read, Write};
use rand::{RngCore, rng};
use hex::{encode, decode};
use log::error;
use std::path::Path;
use serde_json;
use crate::models::{Aes, AesError};

impl Aes {
    pub async fn create_key() -> Key<Aes256Gcm> {
        let mut key = [0u8; 32];
        let mut rng = rng();
        rng.fill_bytes(&mut key);
        Key::<Aes256Gcm>::clone_from_slice(&key)
    }

    /*pub async fn create_hex_key() {
        let key = Self::create_key().await;
        let hex_str = encode(&key);
        let path = Path::new("src/static/fdsgje.txt");
        fs::write(path, hex_str).expect("Не удалось записать в файл");
    }*/

    async fn create_nonce() -> GenericArray<u8, <Aes256Gcm as AeadCore>::NonceSize> {
        let mut key = [0u8; 12];
        let mut rng = rng();
        rng.fill_bytes(&mut key);
        GenericArray::clone_from_slice(&key)
    }

    pub async fn encrypt_data(data: &str) -> Result<Vec<u8>, AesError> {
        dotenv().ok();
        let key_str = env::var("AES_KEY").expect("Добавьте aes key в .env");
        let key_array: [u8; 32] = decode(&key_str).expect("Aes key не в формате hex")
            .as_slice().try_into().expect("Aes key не в формате Vec<u8>, неправильное кол-во символов");
        let key = Key::<Aes256Gcm>::clone_from_slice(&key_array);
        let nonce = Self::create_nonce().await;
        let cipher = Aes256Gcm::new(&key);

        match cipher.encrypt(&nonce, data.as_bytes()) {
            Ok(encrypted) => {
                let mut output = Vec::new();
                output.extend_from_slice(nonce.as_slice());
                output.extend_from_slice(&encrypted);
                return Ok(output)
            },
            Err(e) => {
                error!("Ошибка шифрования aes-gcm: {e}");
                return Err(AesError::EncryptError);
            }
        }
    }

    pub async fn decrypt_data(data: &Vec<u8>) -> Result<String, AesError> {
        dotenv().ok();
        let key_str = env::var("AES_KEY").expect("Добавьте aes key в .env");
        let key_array: [u8; 32] = decode(&key_str).expect("Aes key не в формате hex")
            .as_slice().try_into().expect("Aes key не в формате Vec<u8>, неправильное кол-во символов");
        let key = Key::<Aes256Gcm>::clone_from_slice(&key_array);
        let cipher = Aes256Gcm::new(&key);

        let nonce = GenericArray::from_slice(&data[..12]);
        let mut vec_encrypted = Vec::new();
        vec_encrypted.extend_from_slice(&data[12..]);

        match cipher.decrypt(nonce, vec_encrypted.as_ref()) {
            Ok(decrypted) => {
                return Ok(String::from_utf8(decrypted).expect("Decrypted data не Vec<u8>"))
            },
            Err(e) => {
                error!("Ошибка расшифрования aes-gcm: {e}");
                return Err(AesError::DecryptError)
            }
        }

    }

    pub async fn encrypt_file(path: &Path) -> Result<Vec<u8>, AesError> {
        dotenv().ok();
        let key_hex = env::var("AES_KEY").expect("Добавьте aes key в .env");
        let key_array: [u8; 32] = decode(key_hex).expect("Aes key не в формате hex")
            .as_slice().try_into().expect("Aes key не в формате Vec<u8>, неправильное кол-во символов");
        let key = Key::<Aes256Gcm>::clone_from_slice(&key_array);
        let nonce = Self::create_nonce().await;
        let file = fs::read(path).await.expect("file not found");
        let cipher = Aes256Gcm::new(&key);

        let encrypted = match cipher.encrypt(&nonce, file.as_ref()) {
            Ok(res) => res,
            Err(e) => {
                error!("Ошибка шифрования файла: {e}");
                return Err(AesError::EncryptFileError)
            }
        };

        let mut output = Vec::new();
        output.extend_from_slice(nonce.as_slice());
        output.extend_from_slice(&encrypted);
        Ok(output)
    }

    pub async fn decrypt_file(file: &Vec<u8>) -> Result<Vec<u8>, AesError> {
        dotenv().ok();
        let key_hex = env::var("AES_KEY").expect("Добавьте aes key в .env");
        let key_array: [u8; 32] = decode(key_hex).expect("Aes key не в формате hex")
            .as_slice().try_into().expect("Aes key не в формате Vec<u8>, неправильное кол-во символов");
        let key = Key::<Aes256Gcm>::clone_from_slice(&key_array); 
        let cipher = Aes256Gcm::new(&key);
        let nonce = GenericArray::from_slice(&file[..12]);

        match cipher.decrypt(nonce, &file[12..]) {
            Ok(res) => return Ok(res),
            Err(e) => {
                error!("Ошибка расшифровки файла: {e}");
                return Err(AesError::DecryptFileError)
            }
        }
    }
}