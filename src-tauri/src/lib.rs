//! AgentWatch — entry point.
//!
//! Wiring only: build the store, start the poll loop, expose two commands to the UI.
//! All logic lives in `providers/`, `resolver.rs`, `liveness.rs` and `store.rs`.

pub mod hooks;
mod liveness;
pub mod model;
pub mod providers;
mod resolver;
pub mod store;

use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tauri::Emitter;

use hooks::{event::Decision, server, server::Pending, settings as hook_settings};
use store::{ProviderInfo, SharedStore, Store};

/// Poll cadence. Fast enough that a status change feels instant, cheap because providers
/// only read the bytes appended since the last tick (SPEC-002).
const TICK: Duration = Duration::from_millis(500);

#[tauri::command]
fn get_sessions(store: tauri::State<SharedStore>) -> Vec<model::Session> {
    store.lock().map(|s| s.snapshot()).unwrap_or_default()
}

#[tauri::command]
fn get_providers(store: tauri::State<SharedStore>) -> Vec<ProviderInfo> {
    store.lock().map(|s| s.providers_info()).unwrap_or_default()
}

/// Where AgentWatch keeps its own state: socket, spans, config.
fn data_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Library/Application Support/AgentWatch")
}

fn claude_settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".claude/settings.json")
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct HooksStatus {
    installed: bool,
    settings_path: String,
    backup_path: String,
    backup_exists: bool,
    socket_path: String,
    events: Vec<String>,
}

#[tauri::command]
fn hooks_status() -> HooksStatus {
    let settings = claude_settings_path();
    let backup = hook_settings::backup_path(&settings);
    let installed = hook_settings::describe(&settings)
        .map(|hooks| {
            hooks.values().any(|v| {
                v.as_array()
                    .map(|a| {
                        a.iter().any(|g| {
                            g.get("_source").and_then(|s| s.as_str()) == Some("AgentWatch")
                        })
                    })
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    HooksStatus {
        installed,
        settings_path: settings.display().to_string(),
        backup_path: backup.display().to_string(),
        backup_exists: backup.exists(),
        socket_path: server::socket_path(&data_dir()).display().to_string(),
        events: hook_settings::EVENTS
            .iter()
            .map(|e| e.to_string())
            .collect(),
    }
}

/// Install the hooks. The UI must ask the user first; this command does not.
#[tauri::command]
fn hooks_install() -> Result<String, String> {
    let hook_bin = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("agentwatch-hook")))
        .map(|p| p.display().to_string())
        .ok_or("cannot locate the hook helper next to the app binary")?;

    match hook_settings::install(&claude_settings_path(), &hook_bin)? {
        hook_settings::InstallOutcome::AlreadyInstalled => Ok("already installed".into()),
        hook_settings::InstallOutcome::Installed { backup } => {
            Ok(format!("installed; backup at {}", backup.display()))
        }
    }
}

#[tauri::command]
fn hooks_uninstall() -> Result<bool, String> {
    hook_settings::uninstall(&claude_settings_path())
}

/// Answer the currently blocked turn for a session.
#[tauri::command]
fn answer(store: tauri::State<SharedStore>, session_id: String, decision: String) -> bool {
    let d = match decision.as_str() {
        "allow" => Decision::Allow,
        "deny" => Decision::Deny,
        _ => Decision::Ask,
    };
    store
        .lock()
        .map(|mut s| s.answer(&session_id, d))
        .unwrap_or(false)
}

/// Take back an answer that is still inside its undo window.
#[tauri::command]
fn undo_answer(store: tauri::State<SharedStore>, session_id: String) -> bool {
    store
        .lock()
        .map(|mut s| s.undo(&session_id))
        .unwrap_or(false)
}

/// Sessions whose answer can still be cancelled.
#[tauri::command]
fn undoable(store: tauri::State<SharedStore>) -> Vec<String> {
    store.lock().map(|s| s.undoable()).unwrap_or_default()
}

/// Which sessions are holding a live decision channel right now.
#[tauri::command]
fn awaiting(store: tauri::State<SharedStore>) -> Vec<String> {
    store
        .lock()
        .map(|s| s.awaiting_decision())
        .unwrap_or_default()
}

#[tauri::command]
fn set_idle_after_s(store: tauri::State<SharedStore>, seconds: i64) {
    if let Ok(mut s) = store.lock() {
        s.set_idle_after_s(seconds);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared: SharedStore = Arc::new(Mutex::new(Store::new()));

    tauri::Builder::default()
        .manage(shared.clone())
        .setup(move |app| {
            let handle = app.handle().clone();
            let store = shared.clone();

            // Hook events. A failure to listen must never stop the app: the board still
            // works read-only (that is the whole point of having a file layer at all).
            let (tx, rx) = channel::<Pending>();
            match server::spawn(&data_dir(), tx) {
                Ok(path) => println!("AgentWatch: listening for hooks on {}", path.display()),
                Err(e) => eprintln!("AgentWatch: hook socket unavailable ({e}); read-only mode"),
            }
            let store_for_events = shared.clone();
            thread::spawn(move || {
                for pending in rx {
                    if let Ok(mut s) = store_for_events.lock() {
                        s.apply_hook(pending.event.clone(), pending.reply);
                    }
                }
            });

            // Background poll loop. Deliberately a plain thread: no async runtime needed for
            // a 500 ms timer, and it keeps the resolver free of executor concerns.
            thread::spawn(move || loop {
                thread::sleep(TICK);
                let snapshot = match store.lock() {
                    Ok(mut s) => s.tick().to_vec(),
                    Err(_) => continue,
                };
                let _ = handle.emit("sessions", &snapshot);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_sessions,
            get_providers,
            set_idle_after_s,
            hooks_status,
            hooks_install,
            hooks_uninstall,
            answer,
            undo_answer,
            undoable,
            awaiting
        ])
        .run(tauri::generate_context!())
        .expect("error while running AgentWatch");
}

/// Debug helpers for the live integration test. Print nothing secret: repo, status,
/// confidence and age only. Not used by the app itself.
pub struct DebugRow {
    pub repo: String,
    pub status: String,
    pub confidence: String,
    pub age: String,
}

/// Scan with the app's default window (12h).
pub fn debug_scan() -> Vec<DebugRow> {
    debug_scan_with_window(12)
}

/// Scan with an explicit window in hours. The live test uses a wide window because a
/// machine with nothing running right now legitimately has zero sessions inside 12h.
pub fn debug_scan_with_window(hours: i64) -> Vec<DebugRow> {
    let mut store = Store::with_window(hours);
    let snapshot = store.tick().to_vec();
    snapshot.into_iter().map(row_of).collect()
}

/// Resolve with the app's default settings. Used by integration tests.
pub fn debug_sessions(hours: i64) -> Vec<model::Session> {
    let mut store = Store::with_window(hours);
    store.tick().to_vec()
}

fn row_of(s: model::Session) -> DebugRow {
    DebugRow {
        repo: if s.fact.repo.is_empty() {
            "-".into()
        } else {
            s.fact.repo.clone()
        },
        status: s.status.as_str().to_string(),
        confidence: format!("{:?}", s.confidence).to_lowercase(),
        age: format!("{}s", s.quiet_for_s),
    }
}
