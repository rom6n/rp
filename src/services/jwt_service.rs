use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use chrono::{Duration, Utc};
use uuid::Uuid;
use tokio::fs;
use thiserror::Error;
use std::result::Result;
use log::error;

use crate::models::{Claims, Jwt};

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("JWT error: {0}")]
    JWT(#[from] jsonwebtoken::errors::Error),
    #[error("Error reading key: {0}")]
    ReadingKey(#[from] std::io::Error)
}


async fn access_key() -> Result<String, std::io::Error> {
    fs::read_to_string("ed25519_public.pem").await
}

async fn refresh_key() -> Result<String, std::io::Error> {
    fs::read_to_string("ed25519_private.pem").await
}


impl Jwt {
    pub async fn create_acc_token(id: &str, role: &str) -> Result<String, JwtError> {
        let refresh_key = match refresh_key().await.map_err(|e| JwtError::ReadingKey(e)) {
            Ok(token) => token,
            Err(e) => {
                error!("Ошибка чтения refresh key: {}", e);
                return Err(e)
            }
        };

        let now = Utc::now();
        let claims = Claims {
            sub: format!("{}", id),
            iss: "server".to_owned(),
            aud: "all".to_owned(),
            iat: now.timestamp() as usize,
            exp: (now+Duration::minutes(15)).timestamp() as usize,
            role: role.to_owned(),
            jti: Uuid::new_v4().to_string(),
        };

        let mut header = Header::new(Algorithm::EdDSA);
        header.typ = Some("AccessToken".to_owned());
        encode(&header, &claims, &EncodingKey::from_ed_pem(refresh_key.as_bytes())?).map_err(JwtError::from)
    }






    pub async fn verify_acc_token(token: &str) -> Result<Claims, JwtError> {
        let access_key = match access_key().await.map_err(|e| JwtError::ReadingKey(e)) {
            Ok(token) => token,
            Err(e) => {
                error!("Ошибка чтения access key: {}", e);
                return Err(e)
            }
        };

        let mut val = Validation::new(Algorithm::EdDSA);
        val.set_audience(&["all"]);
        val.set_issuer(&["server"]);
        //val.leeway = 10;
        val.reject_tokens_expiring_in_less_than = 10;
        val.validate_aud = true;

        let data = decode::<Claims>(token, &DecodingKey::from_ed_pem(access_key.as_bytes())?, &val).map_err(JwtError::from);
        match data {
            Ok(claims) => return Ok(claims.claims),
            Err(e) => {
                error!("Ошибка проверки access token: {e}");
                return Err(e)
            }
        }
    }






    pub async fn create_ref_token(id: &str, role: &str) -> Result<String, JwtError> {
        let refresh_key = match refresh_key().await.map_err(|e| JwtError::ReadingKey(e)) {
            Ok(token) => token,
            Err(e) => {
                error!("Ошибка чтения refresh key: {}", e);
                return Err(e)
            }
        };

        let now = Utc::now();
        let claims = Claims {
            sub: format!("{}", id),
            iss: "server".to_owned(),
            aud: "all".to_owned(),
            iat: now.timestamp() as usize,
            exp: (now+Duration::days(30)).timestamp() as usize,
            role: role.to_owned(),
            jti: Uuid::new_v4().to_string(),
        };

        let mut header = Header::new(Algorithm::EdDSA);
        header.typ = Some("RefreshToken".to_owned());
        encode(&header, &claims, &EncodingKey::from_ed_pem(refresh_key.as_bytes())?).map_err(JwtError::from)
    }






    pub async fn verify_ref_token(token: &str) -> Result<Claims, JwtError> {
        let access_key = match access_key().await.map_err(|e| JwtError::ReadingKey(e)) {
            Ok(token) => token,
            Err(e) => {
                error!("Ошибка чтения access key: {}", e);
                return Err(e)
            }
        };

        let mut val = Validation::new(Algorithm::EdDSA);
        val.set_audience(&["all"]);
        val.set_issuer(&["server"]);
        //val.leeway = 10;
        val.reject_tokens_expiring_in_less_than = 20;
        val.validate_aud = true;

        let data = decode::<Claims>(token, &DecodingKey::from_ed_pem(access_key.as_bytes())?, &val).map_err(JwtError::from);
        match data {
            Ok(claims) => return Ok(claims.claims),
            Err(e) => {
                error!("Ошибка проверки access token: {e}");
                return Err(e)
            }
        }
    }


}