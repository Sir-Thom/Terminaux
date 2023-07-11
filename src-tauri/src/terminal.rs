#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize, PtySystem};
use std::{
    io::{BufRead, BufReader, Write},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};
use tauri::{async_runtime::Mutex as AsyncMutex, command, Manager, State, Window};

struct AppState {
    pty_pairs: Arc<AsyncMutex<Vec<PtyPair>>>,
    writers: Arc<AsyncMutex<Vec<Box<dyn Write + Send>>>>,
    next_tty_num: Arc<Mutex<usize>>, // Added next_tty_num field
}

const DEFAULT_ROWS: u16 = 24;
const DEFAULT_COLS: u16 = 80;

const fn new_pty_size() -> PtySize {
    PtySize {
        rows: DEFAULT_ROWS,
        cols: DEFAULT_COLS,
        pixel_width: 0,
        pixel_height: 0,
    }
}

const fn create_new_pty_pair(
    pty_system: &dyn PtySystem,
    tty_num: usize,
) -> Result<PtyPair, std::io::Error> {
    let mut pty_size = new_pty_size();
    pty_size.tty = Some(tty_num);

    pty_system.openpty(pty_size)
}

const fn create_new_writer(pty: &mut PtyPair) -> Result<Box<dyn Write + Send>, std::io::Error> {
    pty.master.take_writer()
}

fn create_pty(pty_system: &dyn PtySystem, next_tty_num: &Arc<Mutex<usize>>) -> Option<PtyPair> {
    let tty_num = {
        let mut next_tty_num = next_tty_num.lock().unwrap();
        let tty_num = *next_tty_num;
        *next_tty_num += 1;
        tty_num
    };

    if let Ok(pty_pair) = create_new_pty_pair(pty_system, tty_num) {
        if let Ok(writer) = create_new_writer(&mut pty_pair) {
            Some(pty_pair)
        } else {
            None
        }
    } else {
        None
    }
}

fn addTab(state: State<'_, AppState>) {
    let mut pty_pairs = state.pty_pairs.lock();
    let mut writers = state.writers.lock();

    let pty_system = native_pty_system();
    let next_tty_num = state.next_tty_num.clone();

    if let Some(pty_pair) = create_pty(&pty_system, &next_tty_num) {
        if let Ok(writer) = create_new_writer(&mut pty_pair) {
            pty_pairs.push(pty_pair);
            writers.push(writer);
        }
    }
}

fn removeTab(tabId: usize, state: State<'_, AppState>) {
    let mut pty_pairs = state.pty_pairs.lock();
    let mut writers = state.writers.lock();

    if tabId < pty_pairs.len() {
        pty_pairs.remove(tabId);
        writers.remove(tabId);
    }
}

#[command]
async fn async_write_to_pty(data: String, state: State<'_, AppState>) -> Result<(), ()> {
    let activeTab = getActiveTab(); // Implement this function to get the index of the active tab
    let writers = state.writers.lock().await;

    if activeTab < writers.len() {
        write!(writers[activeTab].lock().await, "{}", data).map_err(|_| ())?;
    }

    Ok(())
}

#[command]
async fn async_resize_pty(rows: u16, cols: u16, state: State<'_, AppState>) -> Result<(), ()> {
    let activeTab = getActiveTab(); // Implement this function to get the index of the active tab
    let pty_pairs = state.pty_pairs.lock().await;

    if activeTab < pty_pairs.len() {
        pty_pairs[activeTab]
            .lock()
            .await
            .master
            .resize(PtySize {
                rows,
                cols,
                ..Default::default()
            })
            .map_err(|_| ())?;
    }

    Ok(())
}

fn main() {
    let pty_pairs = Arc::new(AsyncMutex::new(Vec::new()));
    let writers = Arc::new(AsyncMutex::new(Vec::new()));

    let reader = {
        let pty_pairs = pty_pairs.clone();
        let writers = writers.clone();

        move |window: Window| {
            let pty_pairs = pty_pairs.clone();
            let writers = writers.clone();
            let window = window.clone();

            thread::spawn(move || {
                let mut readers = Vec::new();
                for pty_pair in pty_pairs.lock().unwrap().iter() {
                    if let Ok(reader) = pty_pair.master.try_clone_reader() {
                        readers.push(Arc::new(Mutex::new(Some(BufReader::new(reader)))));
                    }
                }

                let output = Arc::new(Mutex::new(Vec::new())); // Added output buffer

                loop {
                    for (tabId, reader) in readers.iter().enumerate() {
                        let mut reader = reader.lock().unwrap();
                        if let Some(mut reader) = reader.take() {
                            let mut data = Vec::new();
                            reader.read_to_end(&mut data).unwrap();
                            if !data.is_empty() {
                                output.lock().unwrap().extend(data.iter());
                                window.emit("data", (tabId, data)).unwrap();
                            }
                            *reader = BufReader::new(reader.into_inner());
                        }
                    }
                    sleep(Duration::from_millis(1));
                }
            });
        }
    };

    tauri::Builder::default()
        .manage(AppState {
            pty_pairs: pty_pairs.clone(),
            writers: writers.clone(),
        })
        .on_page_load(move |window, _| {
            reader(window);
        })
        .invoke_handler(tauri::generate_handler![
            addTab,
            removeTab,
            async_write_to_pty,
            async_resize_pty
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
