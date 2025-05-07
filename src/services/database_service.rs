use std::{env, sync::Arc};
use redis::{RedisResult, AsyncCommands};
use deadpool_redis::{Config as RedisConfig, Pool, PoolError};
use sqlx::{PgPool, query, query_as, FromRow, Transaction};
use dotenv::dotenv;
use log::error;
use tokio::task::spawn_blocking;

use crate::models::{Argon, DataBase, DataBaseError, HashExtractDb, Jwt, TimeCustom, User};

impl DataBase {
    pub async fn create_connection() -> PgPool {
        dotenv().ok();
        let db_url = env::var("DATABASE_URL").expect("DATABASE URL must be set");
        let pgpool = PgPool::connect(&db_url).await.expect("Error connecting to database");
        pgpool
    }

    pub async fn verify_ref_token(user_id: &str, token: &str, jti: &str, pool: Arc<PgPool>) -> Result<(), DataBaseError> {
        let req = r#"SELECT token_hash FROM refresh_tokens WHERE user_id = $1 and jti = $2"#;
        match sqlx::query_as::<_, HashExtractDb>(req).bind(user_id.parse::<i64>().unwrap_or(-1)).bind(jti).fetch_one(&*Arc::clone(&pool)).await {
            Ok(res) => match Argon::verify_hash(&res.token_hash, token).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    error!("Ошибка проверки хеша токена: {e}");
                    return  Err(DataBaseError::SomeArgonError(e));
                }
            } 
            Err(e) => {
                error!("Ошибка поиска токена в базе данных: {e}");
                return Err(DataBaseError::NotFound) 
            }
        }
        
    }

    pub async fn del_ref_token(user_id: &str, jti: &str, pool: Arc<PgPool>) -> Result<(), DataBaseError> {
        let req = r#"DELETE FROM refresh_tokens WHERE user_id = $1 and jti = $2 "#;
        let res = query(req).bind(user_id.parse::<i64>().unwrap_or(-1)).bind(jti).execute(&*pool).await;
        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Не удалось удалить использованный refresh токен из БД: {e}");
                return Err(DataBaseError::SqlxError);
            },
        }
    }

    pub async fn save_ref_token(token: &str, pool: Arc<PgPool>) -> Result<(), DataBaseError> {
        if let Ok(claims) = Jwt::verify_ref_token(token, Arc::clone(&pool), false).await {

            let token_hash = match Argon::hash_str(token).await {
                Ok(hash) => hash,
                Err(e) => {
                    error!("Не удалось хешировать refresh токен: {e}");
                    return Err(DataBaseError::SomeArgonError(e));
                }
            };
            
            let expires = match TimeCustom::from_usize_to_timestampz(claims.exp).await {
                Ok(time) => time,
                Err(e) => {
                    error!("Не удалось преобразовать usize expires в timestampz: {e}");
                    return Err(DataBaseError::SomeTimeError(e))
                }
            };

            let created = match TimeCustom::from_usize_to_timestampz(claims.iat).await {
                Ok(time) => time,
                Err(e) => {
                    error!("Не удалось преобразовать usize created в timestampz: {e}");
                    return Err(DataBaseError::SomeTimeError(e))
                }
            };

            let sub: i64 = claims.sub.parse().expect("Не удалось запарсить claims.sub в i64 из usize");

            let req = r#"INSERT INTO refresh_tokens (jti, user_id, token_hash, expires_at, created_at) VALUES ($1, $2, $3, $4, $5)"#;
            match query(req).bind(claims.jti).bind(sub).bind(token_hash).bind(expires).bind(created).execute(&*Arc::clone(&pool)).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    error!("Не удалось сохранить refresh токен в БД: {e}");
                    return Err(DataBaseError::SaveError)
                }
            }
        } else {
            Err(DataBaseError::NonValidToken)
        }
    }

    pub async fn save_user(nickname: &str, name: &str, password: &str, pool: &mut sqlx::Transaction<'static, sqlx::Postgres>) -> Result<User, DataBaseError> {
        let password = match Argon::hash_str(password).await {
            Ok(hash) => hash,
            Err(e) => {
                error!("Не удалось хешировать пароль: {e}");
                return Err(DataBaseError::SomeArgonError(e));
            }
        };

        let req = r#"INSERT INTO users (nickname, name, password) VALUES ($1, $2, $3) RETURNING id, nickname, name, password"#;
        match query_as::<_, User>(req).bind(nickname).bind(name).bind(password).fetch_one(&mut **pool).await {
            Ok(val) => return Ok(val),
            Err(e) => {
                error!("Ошибка добавления нового пользователя в БД: {e}");
                return Err(DataBaseError::SaveError);
            }
        }
    }

    pub async fn get_user(nickname: &str, pool: Arc<PgPool>) -> Result<User, DataBaseError> {
        let req = r#"SELECT * FROM users WHERE nickname = $1"#;

        match query_as::<_, User>(req).bind(nickname).fetch_one(&*Arc::clone(&pool)).await {
            Ok(user) => return Ok(user),
            Err(e) => {
                error!("Не удалось найти пользователя в БД: {e}");
                return  Err(DataBaseError::NotFound);
            }
        }
    }

    pub async fn get_all_users(pool: Arc<PgPool>) -> Result<Vec<User>, DataBaseError> {
        let req = r#"SELECT * FROM users"#;
        let res = query_as::<_, User>(req).fetch_all(&*pool).await;
        match res {
            Ok(users) => return Ok(users),
            Err(e) => {
                error!("Ошибка получения всех пользователей из ДБ: {e}");
                return Err(DataBaseError::SqlxError)
            }
        }
    }

    pub async fn get_user_by_id(id: &str, pool: Arc<PgPool>) -> Result<User, DataBaseError> {
        let req = r#"SELECT * FROM users WHERE id = $1"#;
        let res = query_as::<_, User>(req).bind(id.parse::<i64>().map_err(|_| DataBaseError::SomeError).unwrap()).fetch_one(&*pool).await;
        match res {
            Ok(user) => return Ok(user),
            Err(e) => {
                error!("Ошибка поиска user по id: {e}");
                return Err(DataBaseError::NotFound)
            }
        }
    }
        

}
