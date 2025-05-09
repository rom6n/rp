use axum::{extract::{FromRequest, FromRequestParts}, http::{HeaderMap, request::Parts}};
use cookie::time::error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::{prelude::FromRow, PgPool};
use thiserror::Error;
use argon2::password_hash;

#[derive(Debug, Clone, Copy)]
pub struct ExampleData {
    pub a: i32
}


#[derive(Debug, Clone)]
pub struct Foo(pub &'static str);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct  Claims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct Jwt;

#[derive(Debug, Clone)]
pub struct AuthLayer {
    pub db_conn: Arc<PgPool>,
}

#[derive(Debug, Clone)]
pub struct AuthLayerService<S> {
    pub inner: Option<S>,
    pub db_conn: Arc<PgPool>
}

#[derive(Debug, Clone, Copy)]
pub struct DataBase;

#[derive(Debug, Clone, FromRow)]
pub struct HashExtractDb {
    pub token_hash: String,
}

#[derive(Debug, Clone)]
pub struct Argon;

#[derive(Debug, Error, Clone, Serialize)]
pub enum ArgonError {
    #[error("Error hashing data")]
    HashError,
    #[error("Creating params error")]
    ParamsError,
    #[error("Parse data to hash error")]
    ParseError,
    #[error("Verify hash error")]
    VerifyHashError,
    #[error("Tokio runtime error")]
    TokioError,
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("JWT error: {0}")]
    JWT(#[from] jsonwebtoken::errors::Error),
    #[error("Error reading key: {0}")]
    ReadingKey(#[from] std::io::Error),
    #[error("Error: not found in database")]
    DataBaseNotFound,
}

#[derive(Debug, Clone)]
pub struct  TimeCustom;

#[derive(Debug, Error, Clone, Serialize)]
pub enum TimeCustomError {
    #[error("Parsing error")]
    ParseError,
    #[error("Timestamp error")]
    TimestampError,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterForm {
    pub nickname: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, Error, Serialize)]
pub enum DataBaseError {
    #[error("Argon error: {0}")]
    SomeArgonError(#[from] ArgonError),
    #[error("Save to database error")]
    SaveError,
    #[error("Not found, error")]
    NotFound,
    #[error("Time service error: {0}")]
    SomeTimeError(#[from] TimeCustomError),
    #[error("Non-valid token access/refresh")]
    NonValidToken,
    #[error("Some sqlx error")]
    SqlxError,
    #[error("Some error")]
    SomeError,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub nickname: String,
    pub name: String,
    pub password: String,
}

pub struct Redis;

#[derive(Debug, Clone, Error)]
pub enum CustomRedisError {
    #[error("Some redis error")]
    SomeError
}