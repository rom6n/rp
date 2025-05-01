use axum::{extract::{FromRequest, FromRequestParts}, http::{HeaderMap, request::Parts}};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::{prelude::FromRow, PgPool};

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