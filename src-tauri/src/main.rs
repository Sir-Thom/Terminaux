#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::process::exit;
use std::{
    io::{BufRead, BufReader, Write},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};
use tauri::{async_runtime::Mutex as AsyncMutex, command, Manager, State, Window};
#[macro_use]
extern crate shells;

static mut DEFAULT_ROWS: u16 = 24;
static mut DEFAULT_COLS: u16 = 80;
static mut PIXEL_WIDTH: u16 = 0;
static mut PIXEL_HEIGHT: u16 = 0;

struct AppState {
    pty_pair: AsyncMutex<PtyPair>,
    writer: AsyncMutex<Box<dyn Write + Send>>,
}

#[tauri::command]
async fn async_shell(state: State<'_, AppState>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let cmd = CommandBuilder::new("powershell.exe");
    #[cfg(not(target_os = "windows"))]
    let cmd = CommandBuilder::new("bash");

    let mut child = state
        .pty_pair
        .lock()
        .await
        .slave
        .spawn_command(cmd)
        .map_err(|err| err.to_string())?;

    thread::spawn(move || {
        let status = child.wait().unwrap();
        exit(status.exit_code() as i32)
    });
    Ok(())
}
#[command]
async fn async_write_to_pty(data: String, state: State<'_, AppState>) -> Result<(), ()> {
    write!(state.writer.lock().await, "{}", data).map_err(|_| ())
}

#[command]
async fn async_resize_pty(rows: u16, cols: u16, state: State<'_, AppState>) -> Result<(), ()> {
    state
        .pty_pair
        .lock()
        .await
        .master
        .resize(PtySize {
            rows,
            cols,
            ..Default::default()
        })
        .map_err(|_| ())
}

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
            async_write_to_pty,
            async_resize_pty,
            async_shell
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
