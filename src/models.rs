use axum::{extract::{FromRequest, FromRequestParts}, http::{HeaderMap, request::Parts}};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

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

pub struct Jwt;


#[derive(Debug, Clone)]
pub struct AuthLayer;

#[derive(Debug, Clone)]
pub struct AuthLayerService<S> {
    pub inner: Option<S>
}

#[derive(Debug, Clone, Copy)]
pub struct AuthFromRefresh;

#[derive(Debug, Clone)]
pub struct AuthFromRefreshService<S> {
    pub inner: Option<S>
}

#[derive(Debug, Clone, Copy)]
pub struct UpdateTokens;

#[derive(Debug, Clone)]
pub struct UpdateTokensService<S> {
    pub inner: Option<S>
}
