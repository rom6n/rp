use axum::{extract::{rejection::JsonRejection, ConnectInfo, Extension, Json, Path, Request, State}, 
    http::{header, HeaderMap, StatusCode, Uri}, 
    response::{Html, IntoResponse, Redirect, Response}};
use cookie::Cookie;
use deadpool_redis::Pool;
use http::HeaderValue;
use log::{info, error};
use std::{net::SocketAddr, sync::Arc};
use crate::models::*;
use serde_json::Value;
use axum_extra::extract::{cookie::SameSite, CookieJar};
use sqlx::PgPool;



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

async fn _all_the_things(uri: Uri, _payload: Result<Json<Value>, JsonRejection>) -> impl IntoResponse {
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

pub async fn register(Path(data): Path<RegisterForm>, State((pool, redis_pool)): State<(Arc<PgPool>, Arc<Pool>)>) -> impl IntoResponse {
    let mut transaction: sqlx::Transaction<'static, sqlx::Postgres> = pool.begin().await.expect("Ошибка создания transaction из pool");
    let user = match DataBase::save_user(&data.nickname, &data.name, &data.password, &mut transaction, Arc::clone(&redis_pool)).await {
        Ok(user) => user,
        Err(_) => {
            transaction.rollback().await.expect("Не удалось rollback transaction 1");
            let mut resp: Response = Html("<h1>Error, try again later</h1>").into_response();
            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return resp
        }
    };
    
    let access_token = match Jwt::create_acc_token(&format!("{}", &user.id), "User").await {
        Ok(token) => token,
        Err(_) => {
            transaction.rollback().await.expect("Не удалось rollback transaction 2");
            let mut resp: Response = Html("<h1>Error, try again later</h1>").into_response();
            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return resp
        }
    };
    //info!("Созданный access token: {access_token}");

    let refresh_token = match Jwt::create_ref_token(&format!("{}", &user.id), "User").await {
        Ok(token) => token,
        Err(_) => {
            transaction.rollback().await.expect("Не удалось rollback transaction 3");
            let mut resp: Response = Html("<h1>Error, try again later</h1>").into_response();
            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return resp
        }
    };
    transaction.commit().await.expect("Не удалось commit transaction");

    match DataBase::save_ref_token(&refresh_token, Arc::clone(&pool)).await {
        Ok(_) => (),
        Err(e) => {
            error!("Ошибка сохранения ref token в БД: {e}");
            let mut resp: Response = Html("<h1>Error, try again later</h1>").into_response();
            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return resp
        }
    }
    
    Redis::redis_del(Arc::clone(&redis_pool), "user:all").await.expect("Не удалось удалить all в redis");
    
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

    
    let mut response: Response = Redirect::to("/profile")
        .into_response();
    *response.status_mut() = StatusCode::SEE_OTHER;
    response.headers_mut().append(http::header::SET_COOKIE, HeaderValue::from_str(&access_cookie.to_string())
        .unwrap_or(HeaderValue::from_static("")));
    response.headers_mut().append(http::header::SET_COOKIE, HeaderValue::from_str(&refresh_cookie.to_string())
        .unwrap_or(HeaderValue::from_static("")));

    response


}

pub async fn profile(Path(nickname): Path<String>, State((pool, redis_pool)): State<(Arc<PgPool>, Arc<Pool>)>, extension: Option<Extension<Claims>>) -> impl IntoResponse {
    let user = match DataBase::get_user(&nickname, Arc::clone(&pool), Arc::clone(&redis_pool)).await {
        Ok(user) => user,
        Err(_) => return (StatusCode::NOT_FOUND, Html("<h1>User not found</h1>".to_string()))
    };
    let mut _body = String::new();

    if let Some(claims) = extension {
        if claims.sub == format!("{}", user.id) {
            _body = format!(
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
            _body = format!(
                "<h1>Profile</h1>\n\
                <p><strong>Nickname:</strong> {}</p>\n\
                <p><strong>Name:</strong> {}</p>",
                user.nickname,
                user.name,
            );
        }
    } else {
        _body = format!(
            "<h1>Profile</h1>\n\
            <p><strong>Nickname:</strong> {}</p>\n\
            <p><strong>Name:</strong> {}</p>",
            user.nickname,
            user.name,
        );
    }


    (StatusCode::FOUND, Html(_body))
        
}

pub async fn all_users(State((pool, redis_pool)): State<(Arc<PgPool>, Arc<Pool>)>) -> impl IntoResponse {
    let users = DataBase::get_all_users(Arc::clone(&pool), Arc::clone(&redis_pool)).await;
    match users {
        Ok(users) => {
            return (StatusCode::FOUND, Json(users))
        }
        Err(_) => return (StatusCode::NOT_FOUND, Json(vec![]))
    }
}

pub async fn my_profile(State((pool, redis_pool)): State<(Arc<PgPool>, Arc<Pool>)>, extensions: Option<Extension<Claims>>) -> impl IntoResponse {
    let claims = match extensions {
        Some(claims) => claims,
        None => {
            let mut res = Html("<h1>You're not authorized</h1>").into_response();
            *res.status_mut() = StatusCode::UNAUTHORIZED;
            return res
        }
    };

    let user = DataBase::get_user_by_id(&claims.sub, Arc::clone(&pool), Arc::clone(&redis_pool)).await;
    match user {
        Ok(user) => {
            let mut res = Redirect::to(&format!("/profile/{}", &user.nickname)).into_response();
            *res.status_mut() = StatusCode::SEE_OTHER;
            return res
        },
        Err(_) => {
            let mut res = Html("<h1>Error, try again later</h1>").into_response();
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return res
        }
    }

}

pub async fn login(Path(data): Path<(String, String)>, State((pool, redis_pool)): State<(Arc<PgPool>, Arc<Pool>)>) -> impl IntoResponse {
    let nickname = data.0;
    let passwordd = data.1;
    //println!("{}", &passwordd);

    //Redis::redis_del(Arc::clone(&redis_pool), &format!("user_nick:{}", &nickname)).await.expect("Не удалось удалить пользователя из Redis");
    let user_data = DataBase::get_user(&nickname, Arc::clone(&pool), Arc::clone(&redis_pool)).await;
    let user = match user_data {
        Ok(user) => user,
        Err(_) => {
            let mut resp: Response = Html("<h1>User not found</h1>").into_response();
            *resp.status_mut() = StatusCode::NOT_FOUND;
            return resp
        }
    };

    match Argon::verify_hash(&user.password, &passwordd).await {
        Ok(_) => (),
        Err(_) => {
            let mut resp: Response = Html("<h1>Incorrect password, try again</h1>").into_response();
            *resp.status_mut() = StatusCode::BAD_REQUEST;
            return resp
        }
    }

    let acc_token = match Jwt::create_acc_token(&format!("{}", &user.id), "User").await {
        Ok(token) => token,
        Err(_) => {
            let mut resp: Response = Html("<h1>Error, try again later</h1>").into_response();
            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return resp
        }
    };

    let refresh_token = match Jwt::create_ref_token(&format!("{}", &user.id), "User").await {
        Ok(token) => token,
        Err(_) => {
            let mut resp: Response = Html("<h1>Error, try again later</h1>").into_response();
            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return resp
        }
    };

    match DataBase::save_ref_token(&refresh_token, Arc::clone(&pool)).await {
        Ok(_) => (),
        Err(e) => {
            error!("Ошибка сохранения ref token в БД: {e}");
            let mut resp: Response = Html("<h1>Error, try again later</h1>").into_response();
            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return resp
        }
    }

    let mut acc_cookie = Cookie::new("AccessToken", acc_token);
    acc_cookie.set_http_only(false);
    acc_cookie.set_secure(false);
    acc_cookie.set_path("/");
    acc_cookie.set_same_site(SameSite::Strict);

    let mut ref_cookie = Cookie::new("RefreshToken", refresh_token);
    ref_cookie.set_http_only(false);
    ref_cookie.set_secure(false);
    ref_cookie.set_path("/");
    ref_cookie.set_same_site(SameSite::Strict);


    let mut response: Response = Redirect::to("/profile").into_response();
    *response.status_mut() = StatusCode::SEE_OTHER;
    response.headers_mut().append(http::header::SET_COOKIE, HeaderValue::from_str(&acc_cookie.to_string())
        .unwrap_or(HeaderValue::from_static("")));
    response.headers_mut().append(http::header::SET_COOKIE, HeaderValue::from_str(&ref_cookie.to_string())
        .unwrap_or(HeaderValue::from_static("")));

    response
}


pub async fn cipher_text(Path(text): Path<String>) -> impl IntoResponse {
    let data = match Aes::encrypt_data(&text).await {
        Ok(res) => res,
        Err(e) => {
            return format!("Не удалось зашифровать данные: {e}")
        }
    };

    let decrypted = match Aes::decrypt_data(&data).await {
        Ok(decrypted) => decrypted,
        Err(e) => format!("Не удалось расшифровать данные: {e}")
    };

    format!("Изначальные данные: {text}\n\nЗашифровано: {data:?}\nРасшифровано: {decrypted:?}")
}

pub async fn update_user(State((pool, redis_pool)): State<(Arc<PgPool>, Arc<Pool>)>, Path(data): Path<(String, String)>, extension: Option<Extension<Claims>>) -> impl IntoResponse {
    let claims = match extension {
        Some(claims) => claims,
        None => {
            let mut res = Html("<h1>You're not logged in</h1>").into_response();
            *res.status_mut() = StatusCode::UNAUTHORIZED;
            return res
        }  
    };

    let field = data.0;
    let change_to = data.1;
    let mut transaction = pool.begin().await.expect("Не удалось начать транзакцию"); 

    match DataBase::update_user(&claims.sub, &change_to, &field, &mut transaction).await {
        Ok(_) => transaction.commit().await.expect("Не удалось commit транзакцию"),
        Err(_) => {
            let mut res = Html("<h1>Error, try again later</h1>").into_response();
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return res
        }
    }

    Redis::redis_del(Arc::clone(&redis_pool), &format!("user:{}", &claims.sub)).await.expect("Не удалось удалить из Redis пользователя по id");
    Redis::redis_del(Arc::clone(&redis_pool), "user:all").await.expect("Не удалось удалить из Redis пользователя по nickname");

    let user = match DataBase::get_user_by_id(&claims.sub, Arc::clone(&pool), Arc::clone(&redis_pool)).await {
        Ok(user) => user,
        Err(_) => {
            let mut res = Html("<h1>Error, try again later</h1>").into_response();
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return res
        }
    };

    Redis::redis_del(Arc::clone(&redis_pool), &format!("user_nick:{}", &user.nickname)).await.expect("Не удалось удалить из Redis пользователя по nickname");

    let mut res = Redirect::to(&format!("/profile/{}", user.nickname)).into_response();
    *res.status_mut() = StatusCode::SEE_OTHER;
    res
}

pub async fn logout(extensions: Option<Extension<Claims>>, State(pool): State<Arc<PgPool>>, req: Request) -> impl IntoResponse {
    if let Some(_) = extensions {
        let jar = CookieJar::from_headers(req.headers());
        let refresh_token = Jwt::get_refresh_token(&jar).await;

        let refresh_token_claims = match Jwt::verify_ref_token(&refresh_token, Arc::clone(&pool), false).await {
            Ok(token) => token,
            Err(_) => {
                let mut res = Html("<h1>Error, try again later</h1>").into_response();
                *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                return res
            }
        };

        match DataBase::del_ref_token(&refresh_token_claims.sub, &refresh_token_claims.jti, Arc::clone(&pool)).await {
            Ok(_) => (),
            Err(_) => {
                let mut res = Html("<h1>Error, try again later</h1>").into_response();
                *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                return res
            }
        }
        info!("refresh token удален");
    }

    let mut access_cookie = Cookie::new("AccessToken", "");
    access_cookie.set_max_age(cookie::time::Duration::seconds(-1));

    let mut refresh_cookie = Cookie::new("RefreshToken", "");
    refresh_cookie.set_max_age(cookie::time::Duration::seconds(-1));

    let mut res = Html("<h1>You successfully logout</h1>").into_response();
    *res.status_mut() = StatusCode::OK;
    res.headers_mut().append(http::header::SET_COOKIE, HeaderValue::from_str(&access_cookie.to_string())
        .unwrap_or(HeaderValue::from_static("")));
    res.headers_mut().append(http::header::SET_COOKIE, HeaderValue::from_str(&refresh_cookie.to_string())
        .unwrap_or(HeaderValue::from_static("")));
    res

}