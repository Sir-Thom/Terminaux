use portable_pty::PtyPair;

use std::io::Write;
use tauri::async_runtime::Mutex as AsyncMutex;
pub struct AppState {
    pub pty_pair: AsyncMutex<PtyPair>,
    pub writer: AsyncMutex<Box<dyn Write + Send>>,
}
