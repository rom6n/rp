use axum::{extract::{FromRequest, FromRequestParts}, http::{HeaderMap, request::Parts}};
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
    pub db_conn: PgPool,
}

#[derive(Debug, Clone)]
pub struct AuthLayerService<S> {
    pub inner: Option<S>,
    pub db_conn: PgPool
}

#[derive(Debug, Clone, Copy)]
pub struct DataBase;

#[derive(Debug, Clone, FromRow)]
pub struct HashExtractDb {
    pub token_hash: String,
}

#[derive(Debug, Clone)]
pub struct Argon;

#[derive(Debug, Error)]
pub enum ArgonError {
    #[error("Error hashing data")]
    HashError,
    #[error("Creating params error")]
    ParamsError,
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