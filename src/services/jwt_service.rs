use axum::extract;
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use chrono::{Duration, Utc};
use uuid::Uuid;
use tokio::fs;
use thiserror::Error;
use std::result::Result;
use log::error;
use sqlx::PgPool;

use crate::services::database_service::*;
use crate::models::{Claims, DataBase, Jwt, JwtError};


async fn public_key() -> Result<String, std::io::Error> {
    fs::read_to_string("ed25519_public.pem").await
}

async fn private_key() -> Result<String, std::io::Error> {
    fs::read_to_string("ed25519_private.pem").await
}


impl Jwt {
    pub async fn create_acc_token(id: &str, role: &str) -> Result<String, JwtError> {
        let private_key = match private_key().await.map_err(|e| JwtError::ReadingKey(e)) {
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
        encode(&header, &claims, &EncodingKey::from_ed_pem(private_key.as_bytes())?).map_err(JwtError::from)
    }






    pub async fn verify_acc_token(token: &str) -> Result<Claims, JwtError> {
        let public_key = match public_key().await.map_err(|e| JwtError::ReadingKey(e)) {
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

        let data = decode::<Claims>(token, &DecodingKey::from_ed_pem(public_key.as_bytes())?, &val).map_err(JwtError::from);
        match data {
            Ok(claims) => return Ok(claims.claims),
            Err(e) => {
                error!("Ошибка проверки access token: {e}");
                return Err(e)
            }
        }
    }






    pub async fn create_ref_token(id: &str, role: &str) -> Result<String, JwtError> {
        let private_key = match private_key().await.map_err(|e| JwtError::ReadingKey(e)) {
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
        encode(&header, &claims, &EncodingKey::from_ed_pem(private_key.as_bytes())?).map_err(JwtError::from)
    }






    pub async fn verify_ref_token(token: &str, pool: &PgPool, search_in_db: bool) -> Result<Claims, JwtError> {
        let public_key = match public_key().await.map_err(|e| JwtError::ReadingKey(e)) {
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

        let data = decode::<Claims>(token, &DecodingKey::from_ed_pem(public_key.as_bytes())?, &val).map_err(JwtError::from);
        match data {
            Ok(claims) => {
                if search_in_db == true {
                    match DataBase::verify_ref_token(&claims.claims.sub, token, &claims.claims.jti, pool).await {
                        Ok(_) => (),
                        Err(_) => {
                            error!("Refresh токен не найден в базе данных либо не верен");
                            return Err(JwtError::DataBaseNotFound)},
                    }
                }
                return Ok(claims.claims)},
            Err(e) => {
                error!("Ошибка проверки refresh token: {e}");
                return Err(e)
            }
        }
    }

    pub async fn get_refresh_token(jar: &CookieJar) -> String {
        match jar.get("RefreshToken") {
            Some(val) => return val.to_string(),
            None => return String::new()
        }
    }

    pub async fn get_access_token(jar: &CookieJar) -> String {
        match jar.get("AccessToken") {
            Some(val) => return val.to_string(),
            None => return String::new()
        }
    }


}