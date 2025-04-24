use axum::{body::Body, extract::{Path, Query}, response::{Html, IntoResponse, Json}, routing::{delete, get, post, put}, Router};

pub async fn main_page() -> String {
    "Its main page".to_string()
}

pub async fn greet(Path(path): Path<String>) -> String {
    format!("Hello, {}!", path)
}

pub async fn fallback() -> &'static str {
    "This page isnt exist"
}