mod commands;
mod crypto;

pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                let path = args[1].clone();
                if path.ends_with(".nqtxt") {
                    *commands::PENDING_FILE.lock().unwrap() = Some(path);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::encrypt_text,
            commands::decrypt_data,
            commands::pick_and_read_file,
            commands::pick_save_file,
            commands::write_file,
            commands::load_config,
            commands::save_config,
            commands::close_window,
            commands::toggle_fullscreen,
            commands::pick_save_filter,
            commands::write_raw,
            commands::read_file_path,
            commands::read_pending_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
