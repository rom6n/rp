use rp::models::*;
use std::{path::Path, time::Instant};
use image;

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

#[tokio::test]
async fn encrypt_test() {
    let res = Aes::encrypt_data("Привет мир").await;
    assert!(res.is_ok())
}

#[tokio::test]
async fn decrypt_test() {
    let res = Aes::encrypt_data("Привет мир").await.unwrap();
    let res2 = Aes::decrypt_data(&res).await;
    assert!(res2.is_ok())
}

#[tokio::test]
async fn encrypt_file_test() {
    let path = Path::new("src/static/qrcode.png");
    let res = Aes::encrypt_file(path).await;
    //println!("{:?}", res.clone().unwrap());
    assert!(res.is_ok())
}

#[tokio::test]
async fn decrypt_file_test() {
    let path = Path::new("src/static/qrcode.png");
    let encrypted_file = Aes::encrypt_file(path).await.unwrap();
    let decrypted_file = Aes::decrypt_file(&encrypted_file).await.unwrap();
    let check = image::load_from_memory(&decrypted_file);
    assert!(check.is_ok());
}

