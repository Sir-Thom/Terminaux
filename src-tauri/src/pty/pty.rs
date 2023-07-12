use crate::utils::app_state::AppState;
use portable_pty::PtySize;
use std::io::Write;
use tauri::{command, State};
#[command]
pub async fn write_to_pty(data: String, state: State<'_, AppState>) -> Result<(), ()> {
    write!(state.writer.lock().await, "{}", data).map_err(|_| ())
}
#[command]
pub async fn resize_pty(rows: u16, cols: u16, state: State<'_, AppState>) -> Result<(), ()> {
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
