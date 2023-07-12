#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod pty;
mod shell;
mod utils;
use portable_pty::{native_pty_system, PtySize};
use pty::pty::{resize_pty, write_to_pty};
use shell::shell::async_shell;
use std::{
    io::{BufRead, BufReader},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};
use tauri::async_runtime::Mutex as AsyncMutex;
use utils::app_state::AppState;

fn main() {
    let pty_system = native_pty_system();

    let pty_pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    let reader = pty_pair.master.try_clone_reader().unwrap();
    let writer = pty_pair.master.take_writer().unwrap();

    let reader = Arc::new(Mutex::new(Some(BufReader::new(reader))));
    let output = Arc::new(Mutex::new(String::new())); // Added output buffer
    tauri::Builder::default()
        .manage(AppState {
            pty_pair: AsyncMutex::new(pty_pair),
            writer: AsyncMutex::new(writer),
        })
        .on_page_load(move |window, _| {
            let window = window.clone();
            let reader = reader.clone();
            let output = output.clone();

            thread::spawn(move || {
                let reader = reader.lock().unwrap().take();
                if let Some(mut reader) = reader {
                    loop {
                        sleep(Duration::from_millis(1));
                        let data = reader.fill_buf().unwrap().to_vec();
                        reader.consume(data.len());
                        if data.len() > 0 {
                            output
                                .lock()
                                .unwrap()
                                .push_str(&String::from_utf8_lossy(&data));
                            window.emit("data", data).unwrap();
                        }
                    }
                }
            });
        })
        .invoke_handler(tauri::generate_handler![
            write_to_pty,
            resize_pty,
            async_shell
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
