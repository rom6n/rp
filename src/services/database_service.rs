use std::env;
use redis::{RedisResult, AsyncCommands};
use deadpool_redis::{Config as RedisConfig, Pool, PoolError};
use sqlx::{PgPool, query, query_as, FromRow, Transaction};
use crate::models::{DataBase, HashExtractDb, Argon};
use dotenv::dotenv;
use log::error;




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

    pub async fn save_ref_token(token: &str, jti: &str, pool: &PgPool) {
        let req = r#"INSERT INTO refresh_tokens ()VALUES ("#;
        ggf
    }

}
