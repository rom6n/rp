use std::env;
use redis::{RedisResult, AsyncCommands};
use deadpool_redis::{Config as RedisConfig, Pool, PoolError};
use sqlx::{PgPool, query, query_as, FromRow, Transaction};
use crate::models::{Argon, DataBase, HashExtractDb, Jwt};
use dotenv::dotenv;
use log::error;

use crate::models::TimeCustom;




impl DataBase {
    pub async fn create_connection() -> PgPool {
        dotenv().ok();
        let db_url = env::var("DATABASE_URL").expect("DATABASE URL must be set");
        let pgpool = PgPool::connect(&db_url).await.expect("Error connecting to database");
        pgpool
    }

    pub async fn verify_ref_token(user_id: &str, token: &str, jti: &str, pool: &PgPool) -> Result<(), ()> {
        let req = r#"SELECT token_hash FROM refresh_tokens WHERE user_id = $1 and jti = $2"#;
        if let Ok(res) = sqlx::query_as::<_, HashExtractDb>(req).bind(user_id).bind(jti).fetch_one(&*pool).await {
            if let Ok(_) = Argon::verify_hash(&res.token_hash, token).await {
                return Ok(());
            }
        }
        error!("Refresh токен не найден в базе данных");
        return Err(())
    }

    pub async fn del_ref_token(user_id: &str, jti: &str, pool: &PgPool) {
        let req = r#"DELETE * FROM refresh_tokens WHERE user_id = $1 and jti = $2 "#;
        let res = query(req).bind(user_id).bind(jti).execute(&*pool).await;
        match res {
            Ok(_) => (),
            Err(e) => error!("Не удалось удалить использованный refresh токен из БД: {e}"),
        }
    }

    pub async fn save_ref_token(token: &str, pool: &PgPool) {
        if let Ok(claims) = Jwt::verify_ref_token(token, pool, false).await {

            let token_hash = match Argon::hash_str(token).await {
                Ok(hash) => hash,
                Err(e) => {
                    error!("Не удалось хешировать refresh токен: {e}");
                    return ();
                }
            };
            
            let expires = match TimeCustom::from_usize_to_timestampz(claims.exp).await {
                Ok(time) => time,
                Err(e) => {
                    error!("Не удалось преобразовать usize expires в timestampz: {e}");
                    return ()
                }
            };

            let created = match TimeCustom::from_usize_to_timestampz(claims.iat).await {
                Ok(time) => time,
                Err(e) => {
                    error!("Не удалось преобразовать usize created в timestampz: {e}");
                    return ()
                }
            };

            let req = r#"INSERT INTO refresh_tokens (jti, user_id, token_hash, expires_at, created_at) VALUES ($1, $2, $3, $4, $5)"#;
            match query(req).bind(claims.jti).bind(claims.sub).bind(token_hash).bind(expires).bind(created).execute(&*pool).await {
                Ok(_) => (),
                Err(e) => {
                    error!("Не удалось сохранить refresh токен в БД: {e}");
                    return ()
                }
            }
        }
        
    }

}
