use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::{Config as RedisConfig, Pool, PoolError, redis::RedisError};
use redis::RedisResult;
use axum::extract::State;
use std::{result::Result, sync::Arc};
use log::{error, info};


use crate::models::{Redis, CustomRedisError, User};

impl Redis {
    pub async fn create_connection() -> Pool {
        let conn = RedisConfig::from_url("redis://127.0.0.1:6379");
        let pool = conn.create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .map_err(|e| error!("Ошибка создания пула redis: {e}")).unwrap();
        pool
    }

    pub async fn redis_get(pool: Arc<Pool>, key: &str) -> Result<Vec<User>, CustomRedisError> {
        let mut conn = match pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Ошибка получения соединения redis: {e}");
                return Err(CustomRedisError::ConnectError)
            }
        };

        let data: Option<String> = match conn.get(key).await {
            Ok(Some(data)) => data,
            Ok(None) => {
                error!("Null данные в Redis");
                return Err(CustomRedisError::NoneError);
            },
            Err(e) => {
                error!("Ошибка получения данных из redis: {}", e);
                return Err(CustomRedisError::SomeError);
            }
        };

        let user = match serde_json::from_str(&data.unwrap_or_default()) {
            Ok(user) => user,
            Err(e) => {
                error!("Ошибка десериализации данных из serde_json: {}", e);
                return Err(CustomRedisError::SomeError);
            }
        };
        info!("Данные выгружены из Redis!!");

        Ok(user)
    }

    pub async fn redis_set(pool: Arc<Pool>, key: &str, value: Vec<User>) -> Result<(), CustomRedisError> {
        let mut conn = match pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Ошибка получения соединения redis: {e}");
                return Err(CustomRedisError::ConnectError)
            }
        };

        let value = match serde_json::to_string(&value) {
            Ok(value) => value,
            Err(e) => {
                error!("Ошибка serde_json: {e}");
                return Err(CustomRedisError::SomeError)
            }
        };

        let res: Result<(), RedisError> = conn.set_ex(key, value, 600).await;
        match res {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!("Ошибка установки данных в redis: {}", e);
                return Err(CustomRedisError::SomeError);
            }
        }
    }

    pub async fn redis_del(redis_pool: Arc<Pool>, key: &str) -> Result<(), CustomRedisError> {
        let mut conn = match redis_pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Ошибка получения соединения redis: {e}");
                return Err(CustomRedisError::ConnectError)
            }
        };

        let res: Result<(), RedisError> = conn.del(key).await;
        match res {
            Ok(()) => {
                info!("пользователь удален из redis");
                return Ok(())
            },
            Err(e) => {
                error!("Не удалось удалить из Redis: {e}");
                Err(CustomRedisError::DeleteError)
            }
        }
    }

}

