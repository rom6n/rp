use std::{env, sync::Arc};
use deadpool_redis::Pool;
use sqlx::{PgPool, query, query_as, Transaction};
use dotenv::dotenv;
use log::error;

use crate::models::{Argon, DataBase, DataBaseError, HashExtractDb, Jwt, TimeCustom, User, Redis};

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

    pub async fn save_user(nickname: &str, name: &str, password: &str, pool: &mut sqlx::Transaction<'static, sqlx::Postgres>, redis_pool: Arc<Pool>) -> Result<User, DataBaseError> {
        let password = match Argon::hash_str(password).await {
            Ok(hash) => hash,
            Err(e) => {
                error!("Не удалось хешировать пароль: {e}");
                return Err(DataBaseError::SomeArgonError(e));
            }
        };
        
        let req = r#"INSERT INTO users (nickname, name, password) VALUES ($1, $2, $3) RETURNING id, nickname, name, password"#;
        match query_as::<_, User>(req).bind(nickname).bind(name).bind(password).fetch_one(&mut **pool).await {
            Ok(val) => {
                match Redis::redis_set(Arc::clone(&redis_pool), &format!("user:{}", val.id), vec![val.clone()]).await {
                    Ok(()) => (),
                    Err(e) => {
                        error!("Ошибка вызова redis_set: {e}")
                    }
                }

                match Redis::redis_set(Arc::clone(&redis_pool), &format!("user_nick:{}", val.nickname), vec![val.clone()]).await {
                    Ok(()) => (),
                    Err(e) => {
                        error!("Ошибка вызова redis_set: {e}")
                    }
                }
                return Ok(val)
            },
            Err(e) => {
                error!("Ошибка добавления нового пользователя в БД: {e}");
                return Err(DataBaseError::SaveError);
            }
        }
    }

    pub async fn get_user(nickname: &str, pool: Arc<PgPool>, redis_pool: Arc<Pool>) -> Result<User, DataBaseError> {
        match Redis::redis_get(Arc::clone(&redis_pool), &format!("user_nick:{}", nickname)).await {
            Ok(vec) => {
                return Ok(vec.first().unwrap().to_owned())
            },
            Err(e) => {
                error!("Ошибка поиска user в redis: {e}");
            }
        };

        let req = r#"SELECT * FROM users WHERE nickname = $1"#;

        match query_as::<_, User>(req).bind(nickname).fetch_one(&*Arc::clone(&pool)).await {
            Ok(user) => {
                match Redis::redis_set(Arc::clone(&redis_pool), &format!("user_nick:{}", &user.nickname), vec![user.clone()]).await {
                    Ok(()) => (),
                    Err(e) => {
                        error!("Ошибка вызова redis_set: {e}")
                    }
                }

                return Ok(user)
            },
            Err(e) => {
                error!("Не удалось найти пользователя в БД: {e}");
                return  Err(DataBaseError::NotFound);
            }
        }
    }

    pub async fn get_all_users(pool: Arc<PgPool>, redis_pool: Arc<Pool>) -> Result<Vec<User>, DataBaseError> {
        match Redis::redis_get(Arc::clone(&redis_pool), "user:all").await {
            Ok(vec) => {
                return Ok(vec)
            },
            Err(e) => {
                error!("Ошибка поиска all user в redis: {e}");
            }
        };

        let req = r#"SELECT * FROM users"#;
        let res = query_as::<_, User>(req).fetch_all(&*pool).await;
        match res {
            Ok(users) => {
                match Redis::redis_set(Arc::clone(&redis_pool), "user:all", users.clone()).await {
                    Ok(()) => (),
                    Err(e) => {
                        error!("Ошибка вызова redis_set: {e}")
                    }
                }

                return Ok(users)
            },
            Err(e) => {
                error!("Ошибка получения всех пользователей из ДБ: {e}");
                return Err(DataBaseError::SqlxError)
            }
        }
    }

    pub async fn get_user_by_id(id: &str, pool: Arc<PgPool>, redis_pool: Arc<Pool>) -> Result<User, DataBaseError> {
        match Redis::redis_get(Arc::clone(&redis_pool), &format!("user:{}", id)).await {
            Ok(vec) => {
                return Ok(vec.first().unwrap().to_owned())
            },
            Err(e) => {
                error!("Ошибка поиска user в redis: {e}");
            }
        };

        let req = r#"SELECT * FROM users WHERE id = $1"#;
        let res = query_as::<_, User>(req).bind(id.parse::<i64>().map_err(|_| DataBaseError::SomeError).unwrap()).fetch_one(&*pool).await;
        match res {
            Ok(user) => {
                match Redis::redis_set(Arc::clone(&redis_pool), &format!("user:{}", id), vec![user.clone()]).await {
                    Ok(()) => (),
                    Err(e) => {
                        error!("Ошибка вызова redis_set: {e}")
                    }
                }
                return Ok(user)
            },
            Err(e) => {
                error!("Ошибка поиска user по id: {e}");
                return Err(DataBaseError::NotFound)
            }
        }
    }

    pub async fn update_user(id: &str, data: &str, field: &str, pool: &mut Transaction<'static, sqlx::Postgres>) -> Result<(), DataBaseError> {
        let id = id.trim().parse::<i64>().expect("id должен быть &str");
        let field = match field {
            "nickname" => "nickname",
            "name" => "name",
            "password" => "password",
            e@ _ => {
                error!(r#"Поле "{e}" не существует в TABLE users"#);
                return Err(DataBaseError::SomeError)
            },
        };

        let mut data = data.to_owned();
        if field == "password" {
            data = match Argon::hash_str(&data).await {
                Ok(password) => password,
                Err(e) => return Err(DataBaseError::SomeArgonError(e)) 
            };
        }

        let req = format!("UPDATE users SET {} = '{}' WHERE id = $1", field, data);
        match query(&req).bind(id).execute(&mut **pool).await {
            Ok(_) => {
                return Ok(())
            },
            Err(e) => {
                error!("Ошибка обновления пользователя: {e}");
                return Err(DataBaseError::SqlxError)
            }
        }

    }
        

}
