use rp::models::*;
use std::time::Instant;

#[tokio::test]
async fn create_key_test() {
    let start = Instant::now();
    let key = Aes::create_key().await;
    let stop = start.elapsed().as_millis();
    println!("{:?}\nВремя: {stop}", key);
}

#[tokio::test]
async fn create_hex_key_test() {
    //let start = Instant::now();
    //let key = Aes::create_hex_key().await;
    //let stop = start.elapsed().as_millis();
    //println!("{:?}\nВремя: {stop}", key);
}