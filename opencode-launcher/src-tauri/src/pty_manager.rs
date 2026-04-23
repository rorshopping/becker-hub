use anyhow::Result;
use dashmap::DashMap;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty, Child};
use std::io::{Read, Write};
use std::sync::Arc;
use parking_lot::Mutex;
use tauri::Window;
use regex::Regex;
use once_cell::sync::Lazy;

// Heuristic: detect idle / task completion signals from opencode / generic CLIs
static COMPLETION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)task (complete|finished|done)").unwrap(),
        Regex::new(r"(?i)\u{2713}\s*(done|complete)").unwrap(),
        Regex::new(r"(?i)waiting for input").unwrap(),
        Regex::new(r"(?i)\?\s+$").unwrap(),
        Regex::new(r"\$\s*$").unwrap(),
    ]
});

/// Stores the last N bytes of output per session for persistence.
pub static SCROLLBACK_STORE: Lazy<DashMap<String, String>> = Lazy::new(DashMap::new);
const MAX_SCROLLBACK: usize = 8192;

pub struct PtySession {
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    child: Arc<Mutex<Box<dyn Child + Send + Sync>>>,
    #[allow(dead_code)]
    pub command: String,
    #[allow(dead_code)]
    pub cwd: Option<String>,
}

pub struct PtyManager {
    sessions: DashMap<String, PtySession>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self { sessions: DashMap::new() }
    }

    pub fn spawn(
        &self,
        id: String,
        command: String,
        cwd: Option<String>,
        window: Window,
    ) -> Result<()> {
        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows: 30, cols: 120, pixel_width: 0, pixel_height: 0,
        })?;

        let mut cmd = if cfg!(windows) {
            let mut c = CommandBuilder::new("cmd.exe");
            c.args(["/c", &format!("{} & pause", command)]);
            c
        } else {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
            let mut c = CommandBuilder::new(shell);
            c.args(["-lc", &command]);
            c
        };

        if let Some(ref dir) = cwd { cmd.cwd(dir); }
        for (k, v) in std::env::vars() { cmd.env(k, v); }

        let child = pair.slave.spawn_command(cmd)?;
        let writer = pair.master.take_writer()?;
        let reader = pair.master.try_clone_reader()?;

        let session = PtySession {
            master: Arc::new(Mutex::new(pair.master)),
            writer: Arc::new(Mutex::new(writer)),
            child: Arc::new(Mutex::new(child)),
            command: command.clone(),
            cwd: cwd.clone(),
        };
        self.sessions.insert(id.clone(), session);
        SCROLLBACK_STORE.insert(id.clone(), String::new());

        let id_clone = id.clone();
        let win = window.clone();
        std::thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 4096];
            let mut tail = String::new();
            let mut idle_checker = std::time::Instant::now();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        let _ = win.emit(&format!("pty-exit:{}", id_clone), ());
                        break;
                    }
                    Ok(n) => {
                        let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                        tail.push_str(&chunk);
                        while tail.char_indices().nth(4096).is_some() {
                            if let Some(idx) = tail.char_indices().nth(4096) {
                                tail.drain(..idx.0);
                            } else {
                                break;
                            }
                        }

                        let _ = win.emit(&format!("pty-data:{}", id_clone), &chunk);

                        if idle_checker.elapsed().as_millis() > 500 {
                            for re in COMPLETION_PATTERNS.iter() {
                                if re.is_match(&tail) {
                                    let _ = win.emit("agent-finished", serde_json::json!({
                                        "id": id_clone,
                                        "snippet": tail.chars().rev().take(200).collect::<String>().chars().rev().collect::<String>(),
                                    }));
                                    tail.clear();
                                    break;
                                }
                            }
                            idle_checker = std::time::Instant::now();
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(())
    }

    pub fn write(&self, id: &str, data: &str) -> Result<()> {
        if let Some(s) = self.sessions.get(id) {
            s.writer.lock().write_all(data.as_bytes())?;
        }
        Ok(())
    }

    pub fn resize(&self, id: &str, cols: u16, rows: u16) -> Result<()> {
        if let Some(s) = self.sessions.get(id) {
            s.master.lock().resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })?;
        }
        Ok(())
    }

    pub fn kill(&self, id: &str) -> Result<()> {
        if let Some((_, s)) = self.sessions.remove(id) {
            let _ = s.child.lock().kill();
        }
        SCROLLBACK_STORE.remove(id);
        Ok(())
    }

    /// Get session info for persistence
    #[allow(dead_code)]
    pub fn get_session_info(&self, id: &str) -> Option<(String, Option<String>, String)> {
        self.sessions.get(id).map(|s| {
            let scrollback = SCROLLBACK_STORE
                .get(id)
                .map(|sb| sb.clone())
                .unwrap_or_default();
            (s.command.clone(), s.cwd.clone(), scrollback)
        })
    }

    #[allow(dead_code)]
    pub fn active_ids(&self) -> Vec<String> {
        self.sessions.iter().map(|r| r.key().clone()).collect()
    }
}
