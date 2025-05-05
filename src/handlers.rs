use axum::{body::Body, extract::{rejection::JsonRejection, ConnectInfo, Extension, Json, Path, Query, State}, http::{header, HeaderMap, StatusCode, Uri}, 
    response::{Html, IntoResponse, Redirect, Response}, routing::{delete, get, post, put}, Router};
use cookie::Cookie;
use http::HeaderValue;
use log::info;
use std::{net::SocketAddr, sync::Arc};
use crate::models::*;
use serde_json::Value;
use axum_extra::extract::cookie::SameSite;
use sqlx::PgPool;
use tokio::task::spawn_blocking;



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

pub async fn register(Path(data): Path<RegisterForm>, State(pool): State<Arc<PgPool>>) -> impl IntoResponse {
    let mut transaction: sqlx::Transaction<'static, sqlx::Postgres> = pool.begin().await.expect("Ошибка создания transaction из pool");
    let user = match DataBase::save_user(&data.nickname, &data.name, &data.password, &mut transaction).await {
        Ok(user) => user,
        Err(_) => return {
            transaction.rollback().await.expect("Не удалось rollback transaction");
            Redirect::to("/").into_response()
        }
    };
    
    let access_token = match Jwt::create_acc_token(&format!("{}", &user.id), "User").await {
        Ok(token) => token,
        Err(_) => return {
            transaction.rollback().await.expect("Не удалось rollback transaction");
            Redirect::to("/").into_response()
        }
    };
    //info!("Созданный access token: {access_token}");

    let refresh_token = match Jwt::create_ref_token(&format!("{}", &user.id), "User").await {
        Ok(token) => token,
        Err(_) => return {
            transaction.rollback().await.expect("Не удалось rollback transaction");
            Redirect::to("/").into_response()
        }
    };

    DataBase::save_ref_token(&refresh_token, Arc::clone(&pool)).await;
    transaction.commit().await.expect("Не удалось commit transaction");


    let mut access_cookie = Cookie::new("AccessToken", access_token);
    access_cookie.set_http_only(false);
    access_cookie.set_secure(false);
    access_cookie.set_path("/");
    access_cookie.set_same_site(SameSite::Strict);

    let mut refresh_cookie = Cookie::new("RefreshToken", refresh_token);
    refresh_cookie.set_http_only(false);
    refresh_cookie.set_secure(false);
    refresh_cookie.set_path("/");
    refresh_cookie.set_same_site(SameSite::Strict);

    
    let mut response: Response = Redirect::to(&format!("/profile/{}", user.nickname))
        .into_response();
    *response.status_mut() = StatusCode::SEE_OTHER;
    response.headers_mut().append(http::header::SET_COOKIE, HeaderValue::from_str(&access_cookie.to_string())
        .unwrap_or(HeaderValue::from_static("")));
    response.headers_mut().append(http::header::SET_COOKIE, HeaderValue::from_str(&refresh_cookie.to_string())
        .unwrap_or(HeaderValue::from_static("")));

    response


}

pub async fn profile(Path(nickname): Path<String>, State(pool): State<Arc<PgPool>>, extensions: Option<Extension<Claims>>) -> impl IntoResponse {
    let user = match DataBase::get_user(&nickname, Arc::clone(&pool)).await {
        Ok(user) => user,
        Err(_) => return (StatusCode::NOT_FOUND, Html("User not found".to_string()))
    };
    let mut body = String::new();

    if let Some(claims) = extensions {
        if claims.sub == format!("{}", user.id) {
            body = format!(
                "
                <h1>Your profile</h1>\n\
                <p><strong>Nickname:</strong> {}</p>\n\
                <p><strong>Name:</strong> {}</p>\n\
                <p><strong>ID:</strong> {}</p>\n\
                <p><strong>Password:</strong> {}</p>\n\
                ",
                user.nickname,
                user.name,
                user.id,
                user.password,
            );
        } else {
            body = format!(
                "<h1>Profile</h1>\n\
                <p><strong>Nickname:</strong> {}</p>\n\
                <p><strong>Name:</strong> {}</p>",
                user.nickname,
                user.name,
            );
        }
    } else {
        body = format!(
            "<h1>Profile</h1>\n\
            <p><strong>Nickname:</strong> {}</p>\n\
            <p><strong>Name:</strong> {}</p>",
            user.nickname,
            user.name,
        );
    }


    (StatusCode::FOUND, Html(body))
        
}