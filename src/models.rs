use axum::{extract::{FromRequest, FromRequestParts}, http::{HeaderMap, request::Parts}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct ExampleData {
    pub a: i32
}


#[derive(Debug, Clone)]
pub struct Foo(pub &'static str);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
    pub role: String,
}

pub struct Jwt;

pub struct AuthLayer;