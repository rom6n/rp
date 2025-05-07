use rp::models::*;
use rp::services::*;
use rp::handlers::*;
use pretty_assertions::{assert_eq, assert_ne};
use rand::random;
use sqlx::error::DatabaseError;
use sqlx::Transaction;
use std::sync::Arc;
use log::info;
use std::time::Instant;

#[tokio::test]
async fn test_get_pool() {
    let pool = DataBase::create_connection().await;
    let pool_str = format!("{:?}", pool);
    assert!(!pool_str.is_empty())
}

#[tokio::test]
async fn save_user_test() {
    let random_id: u32 = random();
    let pool = Arc::new(DataBase::create_connection().await);
    let time = Instant::now();

    let mut transaction = pool.begin().await.expect("Не удалось превратить pool в транзакцию");
    let res = DataBase::save_user(&format!("test-user{}", random_id), "test-user", "123", &mut transaction).await;
    
    let stop = time.elapsed().as_millis();
    println!("{:?}, время: {}", res.clone().unwrap(), stop); // User { id: 17, nickname: "test-user688311194", name: "test-user", password: ... }, u128
    assert!(res.is_ok());
    transaction.rollback().await.expect("Не удалось rollback транзакцию");
    println!("Rollback произошел");
}

#[tokio::test]
async fn get_user_test() {
    let pool = Arc::new(DataBase::create_connection().await);
    let res = DataBase::get_user("test-user688311194", pool).await;
    assert!(res.is_ok())
}

#[tokio::test]
async fn save_ref_token_test() {
    let pool = Arc::new(DataBase::create_connection().await);
    let res = DataBase::save_ref_token("non-valid-token", pool).await;
    assert!(res.is_err())
    
}

