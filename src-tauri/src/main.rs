#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use tauri::Emitter;

fn main() {
    let is_autostart = std::env::args().any(|a| a == "--autostart");

    match TcpListener::bind("127.0.0.1:57431") {
        Ok(listener) => {
            std::thread::spawn(move || {
                for mut stream in listener.incoming().flatten() {
                    let mut path = String::new();
                    let _ = stream.read_to_string(&mut path);
                    if let Some(handle) = nq_editor::APP_HANDLE.get()
                        && let Some(window) = nq_editor::ensure_window(handle)
                    {
                        let _ = window.show();
                        let _ = window.set_focus();
                        if !path.is_empty() {
                            let _ = window.emit("open-file", &path);
                        }
                    }
                }
            });
            nq_editor::run(is_autostart);
        }
        Err(_) => {
            let path = std::env::args().nth(1).unwrap_or_default();
            if &path != "--autostart"
                && let Ok(mut stream) = TcpStream::connect("127.0.0.1:57431")
            {
                let _ = stream.write_all(path.as_bytes());
                let _ = stream.shutdown(Shutdown::Write);
            }
        }
    }
}
