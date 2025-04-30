use axum::response;
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
use cookie::{Cookie, SameSite};
use serde_json::{to_string, from_str};
use log::{info, error};
use chrono::{Duration, Utc};

use crate::models::{AuthFromRefresh, AuthFromRefreshService, UpdateTokens, UpdateTokensService, Jwt, Claims};
use crate::services::jwt_service::*;


impl<S> Layer<S> for AuthFromRefresh {
    type Service = AuthFromRefreshService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        AuthFromRefreshService {inner: Some(inner)}
    }
}


impl<S, ResBody, ReqBody> Service<Request<ReqBody>> for AuthFromRefreshService<S> 
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
            Some(next) => next.poll_ready(cx),
            None => panic!("Inner was taken before poll_ready"),
        }
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let auth: String = req
            .headers()
            .get("Authorized")
            .map(|hv| hv.to_str().unwrap_or("true"))
            .unwrap_or("true")
            .to_string();

        let cookies = CookieJar::from_headers(req.headers());

        let mut inner = self.inner.take().expect("Service called after completion");

        Box::pin(async move {
            info!("AuthFromRefresh работает!");

            if auth != "true" {
                let refresh = if let Some(refresh) = cookies.get("RefreshToken") {
                    refresh.to_string()
                } else {
                    String::new()
                };
                match Jwt::verify_ref_token(&refresh).await {
                    Ok(claims) => {
                        req.headers_mut().insert("Claims", 
                        HeaderValue::from_str(&format!("{} {} {}", claims.sub, claims.role, 0 as usize))
                            .unwrap_or(HeaderValue::from_static("")));

                        req.headers_mut().insert("RefreshExp", 
                        HeaderValue::from_str(&format!("{}", claims.exp))
                            .unwrap_or(HeaderValue::from_static("0")));

                        req.headers_mut().insert("Authorized", HeaderValue::from_static("true"));

                    }
                    Err(e) => {
                        error!("Ошибка проверки refresh токена в layer: {e}");
                    }
                }
            }
            
            let response = inner.call(req).await?;
            Ok(response)
        })
    }
}



impl<S> Layer<S> for UpdateTokens {
    type Service = UpdateTokensService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        UpdateTokensService {inner: Some(inner)}
    }
}


impl<S, ResBody, ReqBody> Service<Request<ReqBody>> for UpdateTokensService<S> 
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
            Some(next) => next.poll_ready(cx),
            None => panic!("Inner was taken before poll_ready"),
        }
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let access_exp_str = req.headers().get("AccessExp")
            .map(|val| val.to_str().unwrap_or("0"))
            .unwrap_or("0").to_string();

        let refresh_exp_str = req.headers().get("RefreshExp")
            .map(|val| val.to_str().unwrap_or("0"))
            .unwrap_or("0").to_string();

        let access_exp: usize = access_exp_str.parse().unwrap_or(0 as usize);
        let refresh_exp: usize = refresh_exp_str.parse().unwrap_or(0 as usize);

        let mut inner = self.inner.take().expect("Service called after completion");

        Box::pin(async move {
            info!("UpdateTokens работает!");

            let mut new_access_token = String::new();
            let mut new_refresh_token = String::new();
            let now = Utc::now();

            if access_exp < now.timestamp() as usize {
                if !refresh_exp < now.timestamp() as usize {
                    let jar = CookieJar::from_headers(req.headers());
                    let refresh = if let Some(token) = jar.get("RefreshToken") {
                        token.to_string()
                    } else {
                        String::new()
                    };

                    let verified_refresh = Jwt::verify_ref_token(&refresh).await;

                    if refresh_exp < (now + Duration::days(10)).timestamp() as usize {
                        if let Ok(claims) = &verified_refresh {
                            new_refresh_token = match Jwt::create_ref_token(&claims.sub, &claims.role).await {
                                Ok(token) => token,
                                Err(e) => {
                                    error!("Ошибка создания refresh token: {e}");
                                    String::new()
                                },
                            };
                        }
                    }

                    if let Ok(claims) = &verified_refresh {
                        new_access_token = match Jwt::create_acc_token(&claims.sub, &claims.role).await {
                            Ok(token) => token,
                            Err(e) => {
                                error!("Не удалось создания access token: {e}");
                                let response = inner.call(req).await?;
                                return Ok(response)
                            },
                        };
                    }
                }
            }

            let mut response = inner.call(req).await?;
            
            if !new_access_token.is_empty() {
                let mut acc_cookie = Cookie::new("Authorization", format!("Bearer {}", new_access_token));
                    acc_cookie.set_http_only(true);
                    acc_cookie.set_secure(true);
                    acc_cookie.set_same_site(SameSite::Strict);
                    acc_cookie.set_path("/");
                    acc_cookie.set_max_age(cookie::time::Duration::minutes(15));

                response.headers_mut().append(http::header::SET_COOKIE, 
                    HeaderValue::from_str(&acc_cookie.to_string())
                    .unwrap_or(HeaderValue::from_static("")));
            }
            if !new_refresh_token.is_empty() {
                let mut ref_cookie = Cookie::new("RefreshToken", format!("{}", new_refresh_token));
                ref_cookie.set_http_only(true);
                ref_cookie.set_secure(true);
                ref_cookie.set_same_site(SameSite::Strict);
                ref_cookie.set_path("/");
                ref_cookie.set_max_age(cookie::time::Duration::days(30));

                response.headers_mut().append(http::header::SET_COOKIE, 
                    HeaderValue::from_str(&ref_cookie.to_string())
                    .unwrap_or(HeaderValue::from_static("")));
            }
            
            Ok(response)
        })
    }
}