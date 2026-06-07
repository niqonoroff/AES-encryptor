use std::path::PathBuf;
use std::sync::Mutex;

use base64::Engine;
use crate::crypto;

pub(crate) static PENDING_FILE: Mutex<Option<String>> = Mutex::new(None);

#[tauri::command]
pub async fn encrypt_text(
    text: String,
    password: String,
    argon_time: u32,
    argon_memory: u32,
    argon_parallel: u32,
    salt_size: u32,
    nonce_size: u32,
) -> String {
    let result = tauri::async_runtime::spawn_blocking(move || {
        let encrypted = crypto::encrypt(
            &text, &password,
            argon_time, argon_memory, argon_parallel,
            salt_size as usize, nonce_size as usize,
        );
        base64::engine::general_purpose::STANDARD.encode(&encrypted)
    }).await.unwrap();
    result
}

#[tauri::command]
pub async fn decrypt_data(
    data_b64: String, password: String,
    argon_time: u32, argon_memory: u32, argon_parallel: u32,
    salt_size: u32, nonce_size: u32,
) -> Result<String, String> {
    let data = base64::engine::general_purpose::STANDARD
        .decode(&data_b64).map_err(|e| format!("Base64 error: {}", e))?;
    let result = tauri::async_runtime::spawn_blocking(move || {
        crypto::decrypt(
            &data, &password,
            argon_time, argon_memory, argon_parallel,
            salt_size as usize, nonce_size as usize,
        )
    }).await.unwrap();
    result
}

#[tauri::command]
pub fn pick_and_read_file() -> Result<(String, String), String> {
    let path = rfd::FileDialog::new()
        .add_filter("TXT Encrypted", &["nqtxt"])
        .set_title("Open File")
        .pick_file()
        .ok_or_else(|| "Cancelled".to_string())?;
    let data = std::fs::read(&path).map_err(|e| format!("Read error: {}", e))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    Ok((path.to_string_lossy().to_string(), b64))
}

#[tauri::command]
pub fn pick_save_file() -> Option<String> {
    let p = rfd::FileDialog::new()
        .add_filter("TXT Encrypted", &["nqtxt"])
        .set_title("Save File")
        .save_file()?;
    let s = p.to_string_lossy().to_string();
    if s.ends_with(".nqtxt") { Some(s) } else { Some(s + ".nqtxt") }
}

#[tauri::command]
pub fn write_file(path: String, data_b64: String) -> Result<(), String> {
    let data = base64::engine::general_purpose::STANDARD
        .decode(&data_b64).map_err(|e| format!("Base64 error: {}", e))?;
    std::fs::write(&path, &data).map_err(|e| format!("Write error: {}", e))
}

#[tauri::command]
pub fn write_raw(path: String, data: String) -> Result<(), String> {
    std::fs::write(&path, &data).map_err(|e| format!("Write error: {}", e))
}

fn get_config_dir() -> PathBuf {
    let base = std::env::var("APPDATA")
        .or_else(|_| std::env::var("XDG_CONFIG_HOME"))
        .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.config", h)))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let dir = base.join("nq-editor");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("config.json")
}

#[tauri::command]
pub fn load_config() -> String {
    let path = get_config_dir();
    std::fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string())
}

#[tauri::command]
pub fn save_config(config_json: String) {
    let path = get_config_dir();
    let _ = std::fs::write(&path, &config_json);
}

#[tauri::command]
pub fn pick_save_filter(ext: String, name: Option<String>) -> Option<String> {
    let filter_name = format!(".{} file", ext);
    let mut dialog = rfd::FileDialog::new()
        .add_filter(&filter_name, &[ext.as_str()])
        .set_title("Save File");
    if let Some(ref n) = name {
        dialog = dialog.set_file_name(n);
    }
    let p = dialog.save_file()?;
    let s = p.to_string_lossy().to_string();
    let ext_dot = format!(".{}", ext);
    if s.ends_with(&ext_dot) { Some(s) } else { Some(s + &ext_dot) }
}

#[tauri::command]
pub fn close_window(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
pub fn toggle_fullscreen(window: tauri::Window) {
    let fs = window.is_fullscreen().unwrap_or(false);
    let _ = window.set_fullscreen(!fs);
}

#[tauri::command]
pub fn pick_and_read_any_file() -> Result<(String, String), String> {
    let path = rfd::FileDialog::new()
        .set_title("Select File to Encrypt")
        .pick_file()
        .ok_or_else(|| "Cancelled".to_string())?;
    let data = std::fs::read(&path).map_err(|e| format!("Read error: {}", e))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    Ok((path.to_string_lossy().to_string(), b64))
}

#[tauri::command]
pub fn read_file_path(path: String) -> Result<(String, String), String> {
    let data = std::fs::read(&path).map_err(|e| format!("Read error: {}", e))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    Ok((path, b64))
}

#[tauri::command]
pub fn read_pending_file() -> Result<Option<(String, String)>, String> {
    let path = match PENDING_FILE.lock().unwrap().take() {
        Some(p) => p,
        None => return Ok(None),
    };
    let data = std::fs::read(&path).map_err(|e| format!("Read error: {}", e))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    Ok(Some((path, b64)))
}
