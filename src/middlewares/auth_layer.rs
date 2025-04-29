use axum::{body::Body, response::Response, extract::Request};
use futures_util::future::BoxFuture;
use http::{HeaderValue};
use tower::{Service, Layer};
use std::str::FromStr;
use std::task::{Context, Poll};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::{sync::Mutex};
use axum_extra::extract::cookie::CookieJar;
use cookie::Cookie;
use serde_json::{to_string, from_str};
use log::info;

use crate::models::{AuthLayer, AuthLayerService, Jwt, Claims};
use crate::services::jwt_service::*;


impl<S> Layer<S> for AuthLayer {
    type Service = AuthLayerService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        AuthLayerService {inner: Some(inner)}
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for AuthLayerService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<S::Response, S::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.inner.as_mut() {
            Some(inner) => inner.poll_ready(cx),
            None => panic!("Service polled after completion"),
        }
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let jar = CookieJar::from_headers(req.headers());

        let access_token = if let Some(token) = jar.get("Authorization") {
            if let Some(token_str) = token.to_string().strip_prefix("Bearer ") {
                token_str.to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        
        let mut future = self.inner.take().expect("Service called after completion");

        Box::pin(async move {
            let mut authorized = false;
            let mut claims = String::new();

            if let Ok(val) = Jwt::verify_acc_token(&access_token).await {
                authorized = true;
                claims = if let Ok(serde_claims) = to_string(&format!("{} {} {}", val.sub, val.role, val.exp)) {serde_claims} else {String::new()};
            }
            info!("AuthLayer работает!");
            if authorized && !claims.is_empty() {
                req.headers_mut().insert("Authorized", HeaderValue::from_static("true"));
                req.headers_mut().insert("claims", HeaderValue::from_str(&claims)
                    .unwrap_or(HeaderValue::from_static("")));
            }
            let response = future.call(req).await?;
            Ok(response)
        })
    }
}





