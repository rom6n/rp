use axum::{extract::{FromRequest, FromRequestParts}, http::{HeaderMap, request::Parts}};
#[derive(Debug, Clone, Copy)]
pub struct ExampleData {
    pub a: i32
}


#[derive(Debug, Clone)]
pub struct Foo(pub &'static str);