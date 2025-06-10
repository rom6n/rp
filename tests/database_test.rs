use rp::models::*;
use rand::random;
use std::sync::Arc;
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
    let redis_pool = Arc::new(Redis::create_connection().await);
    let time = Instant::now();

    let mut transaction = pool.begin().await.expect("Не удалось превратить pool в транзакцию");
    let res = DataBase::save_user(&format!("test-user{}", random_id), "test-user", "123", &mut transaction, Arc::clone(&redis_pool)).await;
    
    let stop = time.elapsed().as_millis();
    println!("{:?}, время: {}", res.clone().unwrap(), stop); // User { id: 17, nickname: "test-user688311194", name: "test-user", password: ... }, u128
    assert!(res.is_ok());
    transaction.rollback().await.expect("Не удалось rollback транзакцию");
    println!("Rollback произошел");
}

#[tokio::test]
async fn get_user_test() {
    let time = Instant::now();
    let pool = Arc::new(DataBase::create_connection().await);
    let redis_pool = Arc::new(Redis::create_connection().await);
    let res = DataBase::get_user("test-user688311194", Arc::clone(&pool), Arc::clone(&redis_pool)).await;
    assert!(res.is_ok());
    let stop = time.elapsed().as_millis();
    println!("{:?}, время: {}", res.clone().unwrap(), stop);
}

#[tokio::test]
async fn save_ref_token_test() {
    let pool = Arc::new(DataBase::create_connection().await);
    let res = DataBase::save_ref_token("non-valid-token", pool).await;
    assert!(res.is_err())
    
}

