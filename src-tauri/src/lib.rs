mod commands;
mod crypto;

use std::sync::OnceLock;
use tauri::{
    Emitter, Manager,
    menu::{CheckMenuItem, Menu, MenuBuilder, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::Color,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

fn read_lang() -> String {
    let path = commands::get_config_dir();
    if let Ok(content) = std::fs::read_to_string(path)
        && let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&content)
        && let Some(lang) = cfg.get("lang").and_then(|v| v.as_str())
    {
        return lang.to_string();
    }
    "en".to_string()
}

pub fn ensure_window(app: &impl Manager<tauri::Wry>) -> Option<tauri::WebviewWindow> {
    if let Some(window) = app.get_webview_window("main") {
        return Some(window);
    }
    tauri::webview::WebviewWindowBuilder::new(
        app,
        "main",
        tauri::WebviewUrl::App("index.html".into()),
    )
    .inner_size(1430.0, 740.0)
    .min_inner_size(600.0, 400.0)
    .resizable(true)
    .decorations(true)
    .title("NQ Editor")
    .visible(true)
    .background_color(Color(26, 26, 28, 255))
    .build()
    .ok()
}

fn build_tray_menu(app: &tauri::AppHandle, lang: &str) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let (open_text, auto_text, exit_text) = if lang == "ru" {
        ("Открыть буфер обмена", "Автозапуск", "Выход")
    } else {
        ("Open buffer", "Autostart", "Exit")
    };
    let autostart_on = app.autolaunch().is_enabled().unwrap_or(false);
    let open = MenuItem::with_id(app, "open_buffer", open_text, true, None::<&str>)?;
    let autostart = CheckMenuItem::with_id(
        app,
        "autostart",
        auto_text,
        true,
        autostart_on,
        None::<&str>,
    )?;
    let exit = MenuItem::with_id(app, "exit", exit_text, true, None::<&str>)?;
    let menu = MenuBuilder::new(app)
        .item(&open)
        .item(&autostart)
        .item(&exit)
        .build()?;
    Ok(menu)
}

fn setup_tray(app: &tauri::AppHandle, lang: &str) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_tray_menu(app, lang)?;
    let lang = lang.to_string();

    let mut tray_builder = TrayIconBuilder::new();
    if let Some(icon) = app.default_window_icon() {
        tray_builder = tray_builder.icon(icon.clone());
    }
    tray_builder
        .menu(&menu)
        .tooltip("NQ Editor")
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "open_buffer" => {
                if let Some(window) = ensure_window(app) {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("paste-clipboard", ());
                }
            }
            "autostart" => {
                let checked = app.autolaunch().is_enabled().unwrap_or(false);
                if checked {
                    let _ = app.autolaunch().disable();
                } else {
                    let _ = app.autolaunch().enable();
                }
                if let Some(tray) = app.tray_by_id("main")
                    && let Ok(new_menu) = build_tray_menu(app, &lang)
                {
                    let _ = tray.set_menu(Some(new_menu));
                }
            }
            "exit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = ensure_window(app) {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn ensure_notification_icon(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    let dir = app.path().app_data_dir().ok()?.join("icons");
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join("icon.png");
    std::fs::write(&path, include_bytes!("../icons/icon.png")).ok()?;
    Some(path)
}

pub fn run(is_autostart: bool) {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .setup(move |app| {
            if let Some(path) = std::env::args().nth(1) {
                let path = std::path::PathBuf::from(&path);
                if path.extension().and_then(|s| s.to_str()) == Some("nqtxt")
                    && let Ok(mut guard) = commands::PENDING_FILE.lock()
                {
                    *guard = Some(path);
                }
            }

            if !is_autostart
                && let Some(window) = ensure_window(app.handle())
            {
                let _ = window.set_focus();
            }

            let handle = app.handle().clone();
            let _ = ensure_notification_icon(&handle);
            let _ = APP_HANDLE.set(handle);

            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyE);
            let _ = app
                .global_shortcut()
                .on_shortcut(shortcut, |app, _sh, event| {
                    if event.state == ShortcutState::Pressed
                        && let Some(window) = ensure_window(app)
                    {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.emit("paste-clipboard", ());
                    }
                });

            let lang = read_lang();
            setup_tray(app.handle(), &lang)?;

            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::open_text,
            commands::save_text,
            commands::encrypt_file_cmd,
            commands::decrypt_file_cmd,
            commands::pick_nqtxt,
            commands::pick_save_nqtxt,
            commands::pick_any_file,
            commands::pick_save_filter,
            commands::load_config,
            commands::save_config,
            commands::close_window,
            commands::toggle_fullscreen,
            commands::read_pending_file,
            commands::update_tray_lang,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, _event| {});
}
