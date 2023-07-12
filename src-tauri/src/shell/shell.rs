use portable_pty::CommandBuilder;

extern crate shells;
use crate::utils;

use std::process::exit;
use std::thread::{self};
use tauri::State;
use utils::app_state::AppState;

#[tauri::command]
pub async fn async_shell(state: State<'_, AppState>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let cmd = CommandBuilder::new("powershell.exe");
    #[cfg(target_os = "windows")]
    cmd.env("TERM", "cygwin");
    #[cfg(not(target_os = "windows"))]
    let mut cmd = CommandBuilder::new("bash");
    #[cfg(not(target_os = "windows"))]
    cmd.env("TERM", "xterm-256color");

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
