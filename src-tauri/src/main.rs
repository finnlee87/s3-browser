// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tokio::sync::Mutex;

use once_cell::sync::Lazy;
use s3_operator::S3Operator;

mod s3_operator;

static SINGLETON: Lazy<Mutex<S3Operator>> = Lazy::new(|| {
    Mutex::new(S3Operator::new())
});

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn list_buckets(access_key: &str, secret_key: &str, endpoint: &str) -> Result<Vec<String>, String> {
    println!("{}", "list_buckets");
    let mut s3_opt = SINGLETON.lock().await;
    s3_opt.init(access_key, secret_key, endpoint).await;
    let result = s3_opt.list_buckets().await;
    println!("{}", "list_buckets");
    return result;
}

#[tauri::command]
async fn list_objects(access_key: &str, secret_key: &str, endpoint: &str, bucket: &str) -> Result<Vec<String>, String> {
    let mut s3_opt = SINGLETON.lock().await;
    s3_opt.init(access_key, secret_key, endpoint).await;
    let result = s3_opt.list_objects(bucket).await;
    return result;
}

#[tauri::command]
async fn create_folder(access_key: &str, secret_key: &str, endpoint: &str, path: &str, folder_name: &str) -> Result<(), String> {
    let mut s3_opt = SINGLETON.lock().await;
    s3_opt.init(access_key, secret_key, endpoint).await;
    return s3_opt.create_folder(folder_name, path).await;
}

#[tauri::command]
async fn upload_file(access_key: &str, secret_key: &str, endpoint: &str, folder_path: &str, file_path: &str) -> Result<(), String> {
    let mut s3_opt = SINGLETON.lock().await;
    s3_opt.init(access_key, secret_key, endpoint).await;
    return s3_opt.upload_file(folder_path, file_path).await;
}

#[tauri::command]
async fn delete_object(access_key: &str, secret_key: &str, endpoint: &str, obj_path: &str) -> Result<(), String> {
    let mut s3_opt = SINGLETON.lock().await;
    s3_opt.init(access_key, secret_key, endpoint).await;
    return s3_opt.delete_object(obj_path).await;
}

#[tauri::command]
async fn download_file(access_key: &str, secret_key: &str, endpoint: &str, file_path: &str, target_path: &str) -> Result<(), String> {
    let mut s3_opt = SINGLETON.lock().await;
    s3_opt.init(access_key, secret_key, endpoint).await;
    return s3_opt.download_file(file_path, target_path).await;
}

#[tauri::command]
async fn create_bucket(access_key: &str, secret_key: &str, endpoint: &str, bucket_name: &str, region: &str) -> Result<(), String> {
    let mut s3_opt = SINGLETON.lock().await;
    s3_opt.init(access_key, secret_key, endpoint).await;
    return s3_opt.create_bucket(bucket_name, region).await;
}

#[tauri::command]
async fn delete_bucket(access_key: &str, secret_key: &str, endpoint: &str, bucket_name: &str) -> Result<(), String> {
    let mut s3_opt = SINGLETON.lock().await;
    s3_opt.init(access_key, secret_key, endpoint).await;
    return s3_opt.delete_bucket(bucket_name).await;
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, list_buckets, list_objects, create_folder, upload_file, delete_object, download_file, create_bucket, delete_bucket])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
