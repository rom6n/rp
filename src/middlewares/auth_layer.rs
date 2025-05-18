use axum::{body::Body, response::{Response, Redirect}, extract::Request};
use futures_util::future::BoxFuture;
use http::{HeaderValue};
use sqlx::Database;
use tower::{Service, Layer};
use std::str::FromStr;
use std::task::{Context, Poll};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::{sync::Mutex};
use axum_extra::extract::cookie::CookieJar;
use cookie::{Cookie, SameSite};
use serde_json::{to_string, from_str};
use log::info;
use chrono::{Utc, Duration};
use sqlx::PgPool;

use crate::models::{AuthLayer, AuthLayerService, Claims, DataBase, Jwt};


impl<S> Layer<S> for AuthLayer {
    type Service = AuthLayerService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        AuthLayerService {inner: Some(inner), db_conn: Arc::clone(&self.db_conn)}
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
        let mut future = self.inner.take().expect("Service called after completion");
        let connection_database = Arc::clone(&self.db_conn);

        Box::pin(async move {
            info!("AuthLayer работает!");
            let mut n_access_token = String::new();
            let mut n_refresh_token = String::new();
            let pool = connection_database;
            let access_token = Jwt::get_access_token(&jar).await;
            

            if let Ok(claims) = Jwt::verify_acc_token(&access_token).await {
                req.extensions_mut().insert(claims);
                
            } else {
                let refresh_token = Jwt::get_refresh_token(&jar).await;

                if let Ok((claims, refresh_token)) = all_checks(Arc::clone(&pool), true, &refresh_token).await {
                    n_refresh_token = refresh_token;    
                    let access_token = Jwt::create_acc_token(&claims.sub, &claims.role).await.unwrap_or("".to_owned());

                    if let Ok(access_claims) = Jwt::verify_acc_token(&access_token).await {
                        n_access_token = access_token;
                        req.extensions_mut().insert(access_claims);
                    } 
                } 
            }

            let mut response = future.call(req).await?;

            if !n_access_token.is_empty() {
                let mut cookie = Cookie::new("AccessToken", n_access_token);
                cookie.set_http_only(true);
                cookie.set_secure(false);
                cookie.set_same_site(SameSite::Strict);
                cookie.set_path("/");

                response.headers_mut()
                    .append(http::header::SET_COOKIE, HeaderValue::from_str(&cookie.to_string())
                    .unwrap_or(HeaderValue::from_static("")));
            }

            if !n_refresh_token.is_empty() {
                let mut cookie = Cookie::new("RefreshToken", n_refresh_token);
                cookie.set_http_only(true);
                cookie.set_secure(false);
                cookie.set_same_site(SameSite::Strict);
                cookie.set_path("/"); 

                response.headers_mut()
                    .append(http::header::SET_COOKIE, HeaderValue::from_str(&cookie.to_string())
                    .unwrap_or(HeaderValue::from_static("")));
            }

            Ok(response)
        })
    }
}

async fn all_checks(pool: Arc<PgPool>, search_in_db: bool, refresh_token: &str) -> Result<(Claims, String), String> {
    if let Ok(claims) = Jwt::verify_ref_token(&refresh_token, Arc::clone(&pool), search_in_db).await {
        if let Ok(()) = DataBase::del_ref_token(&claims.sub, &claims.jti, Arc::clone(&pool)).await {

            let refresh_token = Jwt::create_ref_token(&claims.sub, &claims.role).await.unwrap_or("".to_owned());
            DataBase::save_ref_token(&refresh_token, Arc::clone(&pool)).await.unwrap_or(());
            
            match Jwt::verify_ref_token(&refresh_token, Arc::clone(&pool), true).await {
                Ok(claims) => Ok((claims, refresh_token)),
                Err(e) => Err(format!("Error checking refresh token: {e}"))
            }
            
        } else {
            Err("Error deleting refresh token".to_owned())
        }
    } else {
        Err("Error checking refresh troken".to_owned())
    }
}



