use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::crypto::{self, KdfParams, Meta};
use nosleep::{NoSleep, NoSleepType};
use serde::Serialize;
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_notification::NotificationExt;

pub(crate) static PENDING_FILE: Mutex<Option<PathBuf>> = Mutex::new(None);

fn notify_icon() -> Option<String> {
    let handle = crate::APP_HANDLE.get()?;
    let path = handle
        .path()
        .app_data_dir()
        .ok()?
        .join("icons")
        .join("icon.png");
    if path.exists() {
        Some(path.to_string_lossy().to_string())
    } else {
        None
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecryptResult {
    pub meta: Meta,
    pub output_path: String,
}

fn extract_name_ext(path: &Path) -> (String, String) {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string();
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    (name, ext)
}

fn build_params(time: u32, memory: u32, par: u32) -> KdfParams {
    KdfParams {
        time,
        memory,
        parallelism: par,
    }
}

#[tauri::command]
pub async fn open_text(
    path: String,
    password: String,
    argon_time: u32,
    argon_memory: u32,
    argon_parallel: u32,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let blob = std::fs::read(&path).map_err(|e| format!("Read error: {}", e))?;
        let params = build_params(argon_time, argon_memory, argon_parallel);
        let (meta, plaintext) = crypto::decrypt_bytes(&blob, &password, &params, None)
            .map_err(|e| format!("{:?}", e))?;
        if meta.kind != crypto::KIND_TEXT {
            return Err("File is not a text document".to_string());
        }
        String::from_utf8(plaintext).map_err(|_| "Invalid UTF-8 in text file".to_string())
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
pub async fn save_text(
    path: String,
    text: String,
    password: String,
    argon_time: u32,
    argon_memory: u32,
    argon_parallel: u32,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let out_path = PathBuf::from(&path);
        let (name, ext) = extract_name_ext(&out_path);
        let meta = Meta {
            name,
            ext,
            kind: crypto::KIND_TEXT.to_string(),
        };
        let params = build_params(argon_time, argon_memory, argon_parallel);
        let blob = crypto::encrypt_bytes(text.as_bytes(), &password, &params, &meta, None)
            .map_err(|e| format!("{:?}", e))?;
        let tmp = out_path.with_extension("nqtmp");
        std::fs::write(&tmp, &blob).map_err(|e| format!("Write error: {}", e))?;
        std::fs::rename(&tmp, &out_path).map_err(|e| format!("Rename error: {}", e))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
pub async fn encrypt_file_cmd(
    input: String,
    output: String,
    password: String,
    argon_time: u32,
    argon_memory: u32,
    argon_parallel: u32,
) -> Result<(), String> {
    let handle = crate::APP_HANDLE.get().cloned();
    let output_path = PathBuf::from(&output);
    let out = tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let mut no_sleep = NoSleep::new().ok();
        if let Some(ref mut ns) = no_sleep {
            let _ = ns.start(NoSleepType::PreventUserIdleSystemSleep);
        }

        let h = handle.clone();
        let emit = move |pct: u32| {
            if let Some(ref a) = h
                && let Some(w) = a.get_webview_window("main")
            {
                let _ = w.emit(
                    "encrypt-progress",
                    serde_json::json!({"status": "normal", "progress": pct}),
                );
            }
        };

        emit(0);
        let in_path = PathBuf::from(&input);
        let out_path = PathBuf::from(&output);
        let (name, ext) = extract_name_ext(&in_path);
        let meta = Meta {
            name,
            ext,
            kind: crypto::KIND_BINARY.to_string(),
        };
        let params = build_params(argon_time, argon_memory, argon_parallel);
        let result =
            crypto::encrypt_file(&in_path, &out_path, &password, &params, &meta, Some(&emit));
        emit(100);

        if let Some(ref mut ns) = no_sleep {
            let _ = ns.stop();
        }

        result.map_err(|e| format!("{:?}", e))
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?;

    if out.is_ok()
        && let Some(a) = crate::APP_HANDLE.get()
    {
        let mut b = a
            .notification()
            .builder()
            .title("NQ Editor")
            .body(format!("Encrypted: {}", output_path.display()));
        if let Some(icon) = notify_icon() {
            b = b.icon(icon);
        }
        if let Err(e) = b.show() {
            eprintln!("{e}");
        }
    }

    out
}

#[tauri::command]
pub async fn decrypt_file_cmd(
    input: String,
    password: String,
    argon_time: u32,
    argon_memory: u32,
    argon_parallel: u32,
) -> Result<DecryptResult, String> {
    let handle = crate::APP_HANDLE.get().cloned();
    let out = tauri::async_runtime::spawn_blocking(move || -> Result<DecryptResult, String> {
        let mut no_sleep = NoSleep::new().ok();
        if let Some(ref mut ns) = no_sleep {
            let _ = ns.start(NoSleepType::PreventUserIdleSystemSleep);
        }

        let h = handle.clone();
        let emit = move |pct: u32| {
            if let Some(ref a) = h
                && let Some(w) = a.get_webview_window("main")
            {
                let _ = w.emit(
                    "encrypt-progress",
                    serde_json::json!({"status": "normal", "progress": pct}),
                );
            }
        };

        emit(0);
        let in_path = PathBuf::from(&input);
        let blob = std::fs::read(&in_path).map_err(|e| format!("Read error: {}", e))?;
        let params = build_params(argon_time, argon_memory, argon_parallel);
        let (meta, plaintext) = crypto::decrypt_bytes(&blob, &password, &params, Some(&emit))
            .map_err(|e| format!("{:?}", e))?;

        if meta.kind == crypto::KIND_TEXT {
            if let Some(ref mut ns) = no_sleep {
                let _ = ns.stop();
            }
            return Err("TEXT_FILE".to_string());
        }

        let parent = in_path.parent().unwrap_or_else(|| Path::new("."));
        let out_path = parent.join(&meta.name);
        let tmp = out_path.with_extension("nqtmp");
        std::fs::write(&tmp, &plaintext).map_err(|e| format!("Write error: {}", e))?;
        std::fs::rename(&tmp, &out_path).map_err(|e| format!("Rename error: {}", e))?;
        emit(100);

        if let Some(ref mut ns) = no_sleep {
            let _ = ns.stop();
        }

        Ok(DecryptResult {
            meta,
            output_path: out_path.to_string_lossy().to_string(),
        })
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?;

    if let Ok(ref decrypted) = out
        && let Some(a) = crate::APP_HANDLE.get()
    {
        let mut b = a
            .notification()
            .builder()
            .title("NQ Editor")
            .body(format!("Decrypted: {}", decrypted.output_path));
        if let Some(icon) = notify_icon() {
            b = b.icon(icon);
        }
        if let Err(e) = b.show() {
            eprintln!("{e}");
        }
    }

    out
}

#[tauri::command]
pub fn pick_nqtxt() -> Option<String> {
    let p = rfd::FileDialog::new()
        .add_filter("NQ Encrypted", &["nqtxt"])
        .set_title("Open File")
        .pick_file()?;
    Some(p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn pick_save_nqtxt(suggested: Option<String>) -> Option<String> {
    let mut dialog = rfd::FileDialog::new()
        .add_filter("NQ Encrypted", &["nqtxt"])
        .set_title("Save File");
    if let Some(ref n) = suggested {
        dialog = dialog.set_file_name(n);
    }
    let p = dialog.save_file()?;
    let s = p.to_string_lossy().to_string();
    if s.ends_with(".nqtxt") {
        Some(s)
    } else {
        Some(s + ".nqtxt")
    }
}

#[tauri::command]
pub fn pick_any_file() -> Option<String> {
    let p = rfd::FileDialog::new()
        .set_title("Select File to Encrypt")
        .pick_file()?;
    Some(p.to_string_lossy().to_string())
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
    if s.ends_with(&ext_dot) {
        Some(s)
    } else {
        Some(s + &ext_dot)
    }
}

#[tauri::command]
pub fn update_tray_lang(lang: String) -> Result<(), String> {
    let handle = crate::APP_HANDLE.get().ok_or("no app handle")?.clone();
    let Some(tray) = handle.tray_by_id("main") else {
        return Ok(());
    };

    let (open_text, auto_text, exit_text) = if lang == "ru" {
        ("Открыть буфер обмена", "Автозапуск", "Выход")
    } else {
        ("Open buffer", "Autostart", "Exit")
    };

    let autostart_on = handle.autolaunch().is_enabled().unwrap_or(false);
    let open =
        tauri::menu::MenuItem::with_id(&handle, "open_buffer", open_text, true, None::<&str>)
            .map_err(|e| e.to_string())?;
    let autostart = tauri::menu::CheckMenuItem::with_id(
        &handle,
        "autostart",
        auto_text,
        true,
        autostart_on,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let exit = tauri::menu::MenuItem::with_id(&handle, "exit", exit_text, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let menu = tauri::menu::MenuBuilder::new(&handle)
        .item(&open)
        .item(&autostart)
        .item(&exit)
        .build()
        .map_err(|e| e.to_string())?;
    let _ = tray.set_menu(Some(menu));
    Ok(())
}

pub(crate) fn get_config_dir() -> PathBuf {
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
pub fn close_window(window: tauri::Window) {
    let _ = window.hide();
}

#[tauri::command]
pub fn toggle_fullscreen(window: tauri::Window) {
    let fs = window.is_fullscreen().unwrap_or(false);
    let _ = window.set_fullscreen(!fs);
}

#[tauri::command]
pub fn read_pending_file() -> Option<String> {
    let mut guard = PENDING_FILE.lock().ok()?;
    guard.take().map(|p| p.to_string_lossy().to_string())
}
