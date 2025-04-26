use axum::{body::Body, extract::{rejection::JsonRejection, ConnectInfo, Extension, Json, Path, Query}, http::{header, HeaderMap, StatusCode, Uri}, response::{Html, IntoResponse}, routing::{delete, get, post, put}, Router};
use std::{net::SocketAddr};
use crate::models::*;
use serde_json::Value;


pub async fn main_page() -> String {
    "Its main page".to_string()
}

pub async fn greet(Path(path): Path<String>, ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
    format!("Hello, {}!\naddr: {}", path, addr)
}

pub async fn fallback() -> &'static str {
    "This page isnt exist"
}

pub async fn method_fallback() -> &'static str {
    "This method not allowed"
}

async fn all_the_things(uri: Uri, payload: Result<Json<Value>, JsonRejection>) -> impl IntoResponse {
    let mut header_map = HeaderMap::new();
    if uri.path() == "/" {
        header_map.insert(header::SERVER, "axum".parse().unwrap());
    }

    (
        // set status code
        StatusCode::NOT_FOUND,
        // headers with an array
        [("x-custom", "custom")],
        // some extensions
        Extension(Foo("foo")),
        Extension(Foo("bar")),
        // more headers, built dynamically
        header_map,
        // and finally the body
        "foo",
    )
}

