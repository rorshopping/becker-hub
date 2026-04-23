#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod pty_manager;
mod ai_dictionary;

use pty_manager::PtyManager;
use std::sync::Arc;
use tauri::State;
use serde::{Deserialize, Serialize};

struct AppState {
    pty_manager: Arc<PtyManager>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(dead_code)]
struct PersistedTab {
    id: String,
    name: String,
    slots: Vec<Option<PersistedSlot>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(dead_code)]
struct PersistedSlot {
    command: String,
    cwd: Option<String>,
    scrollback: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[allow(dead_code)]
struct PersistedSettings {
    default_command: String,
}

fn session_file() -> std::path::PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("opencode-launcher");
    std::fs::create_dir_all(&dir).ok();
    dir.join("session.json")
}

#[tauri::command]
fn load_session() -> Result<String, String> {
    let path = session_file();
    if path.exists() {
        std::fs::read_to_string(&path).map_err(|e| e.to_string())
    } else {
        Ok("null".to_string())
    }
}

#[tauri::command]
fn save_session(data: String) -> Result<(), String> {
    let path = session_file();
    std::fs::write(&path, data).map_err(|e| e.to_string())
}

#[tauri::command]
async fn spawn_cli(
    state: State<'_, AppState>,
    window: tauri::Window,
    tab_id: String,
    slot: u32,
    command: String,
    cwd: Option<String>,
) -> Result<String, String> {
    let id = format!("{}-{}", tab_id, slot);
    state.pty_manager
        .spawn(id.clone(), command, cwd, window)
        .map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
fn write_to_pty(state: State<AppState>, id: String, data: String) -> Result<(), String> {
    state.pty_manager.write(&id, &data).map_err(|e| e.to_string())
}

#[tauri::command]
fn resize_pty(state: State<AppState>, id: String, cols: u16, rows: u16) -> Result<(), String> {
    state.pty_manager.resize(&id, cols, rows).map_err(|e| e.to_string())
}

#[tauri::command]
fn kill_pty(state: State<AppState>, id: String) -> Result<(), String> {
    state.pty_manager.kill(&id).map_err(|e| e.to_string())
}

fn main() {
    let pty_manager = Arc::new(PtyManager::new());
    let state = AppState { pty_manager };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            spawn_cli,
            write_to_pty,
            resize_pty,
            kill_pty,
            load_session,
            save_session,
        ])
        .run(tauri::generate_context!())
        .expect("error running app");
}
