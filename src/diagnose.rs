use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use chrono::Local;

struct DiagnoseState {
    path: PathBuf,
    file: Mutex<File>,
}

static DIAGNOSE_STATE: OnceLock<DiagnoseState> = OnceLock::new();

pub fn init() -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join("codex-usage-taskbar.log");
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Unable to open diagnostic log file {}: {e}", path.display()))?;

    let _ = DIAGNOSE_STATE.set(DiagnoseState {
        path: path.clone(),
        file: Mutex::new(file),
    });

    log("diagnostic logging enabled");
    log("session start");
    Ok(path)
}

pub fn is_enabled() -> bool {
    DIAGNOSE_STATE.get().is_some()
}

pub fn log_path() -> PathBuf {
    DIAGNOSE_STATE
        .get()
        .map(|state| state.path.clone())
        .unwrap_or_else(|| std::env::temp_dir().join("codex-usage-taskbar.log"))
}

pub fn log(message: impl AsRef<str>) {
    let Some(state) = DIAGNOSE_STATE.get() else {
        return;
    };

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

    if let Ok(mut file) = state.file.lock() {
        let _ = writeln!(file, "[{timestamp}] {}", message.as_ref());
        let _ = file.flush();
    }
}

pub fn log_error(context: &str, error: impl std::fmt::Display) {
    log(format!("{context}: {error}"));
}
