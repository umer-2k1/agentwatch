//! AgentWatch — entry point and Tauri command bridge.
//!
//! Wires together:
//! * `store`       — single source of truth (sessions, Inbox cards, 2s undo, turns, collisions)
//! * `activity`    — 12-month heatmap, day lanes, replay, leverage, waits, models, records
//! * `collisions`  — file-edit collision detector & advisory file locks
//! * `pty_manager` — embedded PTY session spawner & git worktree isolator
//! * `hosts`       — 30-agent catalog, 13-host capability prober, jump-to-host
//! * `mcp`         — 8-tool MCP JSON-RPC socket server + `~/.claude.json` registration
//! * `iphone`      — local LAN companion HTTP server (`:47811`) + QR pairing token
//! * `tray`        — native macOS menu-bar icon, counts & navigation menu

pub mod activity;
pub mod collisions;
pub mod hooks;
pub mod hosts;
pub mod iphone;
mod liveness;
pub mod mcp;
pub mod model;
pub mod providers;
pub mod pty_manager;
mod resolver;
pub mod store;
mod tray;

use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tauri::{Emitter, Manager};

use hooks::{
    event::Decision,
    server::{self, Pending},
    settings as hook_settings,
};
use iphone::{IphoneServer, IphoneStatus};
use store::{ProviderInfo, SharedStore, Store};

const TICK: Duration = Duration::from_millis(500);

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
fn get_sessions(store: tauri::State<SharedStore>) -> Vec<model::Session> {
    store.lock().map(|s| s.snapshot()).unwrap_or_default()
}

#[tauri::command]
fn get_providers(store: tauri::State<SharedStore>) -> Vec<ProviderInfo> {
    store.lock().map(|s| s.providers_info()).unwrap_or_default()
}

#[tauri::command]
fn get_inbox_cards(store: tauri::State<SharedStore>) -> Vec<model::InboxCard> {
    store.lock().map(|s| s.inbox_cards()).unwrap_or_default()
}

#[tauri::command]
fn get_history_turns(store: tauri::State<SharedStore>) -> Vec<model::TurnRecord> {
    store.lock().map(|s| s.history_turns()).unwrap_or_default()
}

#[tauri::command]
fn get_activity_report(store: tauri::State<SharedStore>) -> Option<activity::ActivityReport> {
    store.lock().ok().map(|s| s.activity_report())
}

#[tauri::command]
fn get_agent_catalog() -> Vec<hosts::AgentCatalogItem> {
    hosts::agent_catalog()
}

#[tauri::command]
fn get_host_catalog() -> Vec<hosts::HostAppItem> {
    hosts::host_catalog()
}

#[tauri::command]
fn get_diagnostics(iphone: tauri::State<Arc<Mutex<IphoneServer>>>) -> hosts::DiagnosticsReport {
    let settings = claude_settings_path();
    let hook_installed = hook_settings::describe(&settings)
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
    let mcp_reg = mcp::is_mcp_registered(&mcp::claude_mcp_path());
    let (oc_exists, oc_schema) = hosts::probe_opencode_schema();
    let (cmux_inst, cmux_probe) = hosts::probe_cmux();
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let codex_exists = PathBuf::from(&home).join(".codex/sessions").exists();
    let claude_exists = PathBuf::from(&home).join(".claude").exists();
    let tmux_exists = PathBuf::from("/opt/homebrew/bin/tmux").exists()
        || PathBuf::from("/usr/local/bin/tmux").exists();
    let sock_ok = server::socket_path(&data_dir()).exists();
    let iph_on = iphone.lock().map(|s| s.status().enabled).unwrap_or(false);

    hosts::DiagnosticsReport {
        claude_installed: claude_exists,
        claude_hook_installed: hook_installed,
        claude_mcp_registered: mcp_reg,
        opencode_db_exists: oc_exists,
        opencode_schema: oc_schema,
        codex_sessions_exist: codex_exists,
        cmux_installed: cmux_inst,
        cmux_cli_probe: cmux_probe,
        tmux_installed: tmux_exists,
        hook_socket_listening: sock_ok,
        iphone_server_running: iph_on,
    }
}

#[tauri::command]
fn who_edits(store: tauri::State<SharedStore>, path: String) -> Option<collisions::WhoEditsReport> {
    store.lock().ok().map(|s| s.who_edits(&path))
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

#[tauri::command]
fn mcp_status() -> bool {
    mcp::is_mcp_registered(&mcp::claude_mcp_path())
}

#[tauri::command]
fn mcp_toggle(enable: bool) -> Result<bool, String> {
    let mcp_bin = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("agentwatch-mcp")))
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "/Applications/AgentWatch.app/Contents/MacOS/agentwatch-mcp".into());
    mcp::set_mcp_registered(&mcp::claude_mcp_path(), enable, &mcp_bin)
}

#[tauri::command]
fn iphone_status(iphone: tauri::State<Arc<Mutex<IphoneServer>>>) -> Option<IphoneStatus> {
    iphone.lock().ok().map(|s| s.status())
}

#[tauri::command]
fn iphone_toggle(
    iphone: tauri::State<Arc<Mutex<IphoneServer>>>,
    enabled: bool,
) -> Option<IphoneStatus> {
    iphone.lock().ok().map(|mut s| s.set_enabled(enabled))
}

#[tauri::command]
fn iphone_rotate_token(iphone: tauri::State<Arc<Mutex<IphoneServer>>>) -> Option<IphoneStatus> {
    iphone.lock().ok().map(|mut s| s.rotate_token())
}

#[tauri::command]
fn iphone_set_relay(
    iphone: tauri::State<Arc<Mutex<IphoneServer>>>,
    relay_url: String,
) -> Option<IphoneStatus> {
    iphone.lock().ok().map(|mut s| s.set_relay_url(relay_url))
}

/// Answer the currently blocked turn for a session (with 2-second undo).
#[tauri::command]
fn answer(
    store: tauri::State<SharedStore>,
    session_id: String,
    decision: String,
    label: Option<String>,
) -> bool {
    let d = if decision == "allow" {
        Decision::Allow
    } else if decision == "deny" {
        Decision::Deny
    } else if let Some(idx_s) = decision.strip_prefix("choose:") {
        let index = idx_s.parse::<usize>().unwrap_or(0);
        Decision::Choose { index }
    } else {
        Decision::Ask
    };
    store
        .lock()
        .map(|mut s| s.answer_with_label(&session_id, d, label))
        .unwrap_or(false)
}

#[tauri::command]
fn undo_answer(store: tauri::State<SharedStore>, session_id: String) -> bool {
    store
        .lock()
        .map(|mut s| s.undo(&session_id))
        .unwrap_or(false)
}

#[tauri::command]
fn undoable(store: tauri::State<SharedStore>) -> Vec<String> {
    store.lock().map(|s| s.undoable()).unwrap_or_default()
}

#[tauri::command]
fn awaiting(store: tauri::State<SharedStore>) -> Vec<String> {
    store
        .lock()
        .map(|s| s.awaiting_decision())
        .unwrap_or_default()
}

#[tauri::command]
fn mark_seen(store: tauri::State<SharedStore>, session_id: String) {
    if let Ok(mut s) = store.lock() {
        s.mark_seen(&session_id);
    }
}

#[tauri::command]
fn mark_all_seen(store: tauri::State<SharedStore>) {
    if let Ok(mut s) = store.lock() {
        s.mark_all_seen();
    }
}

#[tauri::command]
fn send_message(
    store: tauri::State<SharedStore>,
    session_id: String,
    text: String,
    when_done: bool,
) -> Result<String, String> {
    let mut s = store.lock().map_err(|_| "store lock poisoned".to_string())?;
    s.send_or_queue_message(&session_id, &text, when_done)
}

#[tauri::command]
fn spawn_session(
    store: tauri::State<SharedStore>,
    kind: String,
    folder: String,
    prompt: String,
    create_worktree: bool,
) -> Result<pty_manager::SpawnedView, String> {
    let s = store.lock().map_err(|_| "store lock poisoned".to_string())?;
    s.pty
        .spawn(&kind, &folder, &prompt, create_worktree, store::now_ms())
}

#[tauri::command]
fn send_pty_input(
    store: tauri::State<SharedStore>,
    session_id: String,
    text: String,
) -> Result<(), String> {
    let s = store.lock().map_err(|_| "store lock poisoned".to_string())?;
    s.pty.send_input(&session_id, &text)
}

#[tauri::command]
fn stop_spawned_session(store: tauri::State<SharedStore>, session_id: String) -> bool {
    store
        .lock()
        .map(|s| s.pty.stop(&session_id))
        .unwrap_or(false)
}

#[tauri::command]
fn get_spawned_sessions(store: tauri::State<SharedStore>) -> Vec<pty_manager::SpawnedView> {
    store
        .lock()
        .map(|s| s.spawned_sessions())
        .unwrap_or_default()
}

#[tauri::command]
fn simulate_prompt(store: tauri::State<SharedStore>, kind: String) -> String {
    store
        .lock()
        .map(|mut s| s.simulate_prompt(&kind))
        .unwrap_or_default()
}

#[tauri::command]
fn jump_to_agent(place: String, cwd: String) -> Result<String, String> {
    hosts::jump_to_host(&place, &cwd)
}

#[tauri::command]
fn focus_agent(app: tauri::AppHandle, session_id: String) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
    let _ = app.emit("focus-agent", session_id);
}

#[tauri::command]
fn set_strip_position(app: tauri::AppHandle, position: String) {
    place_strip(&app, &position);
}

#[tauri::command]
fn set_idle_after_s(store: tauri::State<SharedStore>, seconds: i64) {
    if let Ok(mut s) = store.lock() {
        s.set_idle_after_s(seconds);
    }
}

fn place_strip(app: &tauri::AppHandle, position: &str) {
    let Some(strip) = app.get_webview_window("strip") else {
        return;
    };
    if position == "menubar" || position == "dock-only" {
        let _ = strip.hide();
        return;
    }
    let monitor = strip
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());
    if let Some(m) = monitor {
        let scale = m.scale_factor();
        let mw = m.size().width as f64 / scale;
        let mh = m.size().height as f64 / scale;
        let (w, h, x, y) = match position {
            "left" => (28.0, 180.0, 4.0, (mh - 180.0) / 2.0),
            "notch" => (180.0, 28.0, (mw - 180.0) / 2.0, 4.0),
            "dock" => (180.0, 28.0, (mw - 180.0) / 2.0, mh - 86.0),
            _ => (28.0, 180.0, mw - 30.0, (mh - 180.0) / 2.0),
        };
        let _ = strip.set_size(tauri::PhysicalSize::new(
            (w * scale) as u32,
            (h * scale) as u32,
        ));
        let _ = strip.set_position(tauri::PhysicalPosition::new(
            (x * scale) as i32,
            (y * scale) as i32,
        ));
        let _ = strip.show();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared: SharedStore = Arc::new(Mutex::new(Store::new()));
    let iphone_server = Arc::new(Mutex::new(IphoneServer::new(shared.clone())));

    tauri::Builder::default()
        .manage(shared.clone())
        .manage(iphone_server)
        .setup(move |app| {
            let handle = app.handle().clone();
            let store = shared.clone();

            if let Ok(mut st) = shared.lock() {
                let snap = st.tick().to_vec();
                match tray::build(&handle, &snap) {
                    Ok(()) => eprintln!("AgentWatch: tray created"),
                    Err(e) => eprintln!("AgentWatch: TRAY FAILED: {e}"),
                }
            }

            app.on_menu_event(|app, event| tray::on_menu_event(app, event));
            place_strip(&handle, "right");

            // Start the hook event socket listener.
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

            // Start the MCP server socket listener (`mcp.sock`).
            if let Ok(path) = mcp::spawn(&data_dir(), shared.clone()) {
                println!("AgentWatch: MCP server on {}", path.display());
            }

            // 500 ms background poll loop.
            thread::spawn(move || loop {
                thread::sleep(TICK);
                let snapshot = match store.lock() {
                    Ok(mut s) => s.tick().to_vec(),
                    Err(_) => continue,
                };
                let _ = handle.emit("sessions", &snapshot);
                tray::update(&handle, &snapshot);
            });

            if let Some(w) = app.get_webview_window("main") {
                let w2 = w.clone();
                w.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w2.hide();
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_sessions,
            get_providers,
            get_inbox_cards,
            get_history_turns,
            get_activity_report,
            get_agent_catalog,
            get_host_catalog,
            get_diagnostics,
            who_edits,
            set_idle_after_s,
            hooks_status,
            hooks_install,
            hooks_uninstall,
            mcp_status,
            mcp_toggle,
            iphone_status,
            iphone_toggle,
            iphone_rotate_token,
            iphone_set_relay,
            answer,
            undo_answer,
            undoable,
            awaiting,
            mark_seen,
            mark_all_seen,
            send_message,
            spawn_session,
            send_pty_input,
            stop_spawned_session,
            get_spawned_sessions,
            simulate_prompt,
            jump_to_agent,
            focus_agent,
            set_strip_position
        ])
        .run(tauri::generate_context!())
        .expect("error while running AgentWatch");
}

/// Debug helpers for the live integration tests.
pub struct DebugRow {
    pub repo: String,
    pub status: String,
    pub confidence: String,
    pub age: String,
}

pub fn debug_scan() -> Vec<DebugRow> {
    debug_scan_with_window(48)
}

pub fn debug_scan_with_window(hours: i64) -> Vec<DebugRow> {
    let mut store = Store::with_window(hours);
    let snapshot = store.tick().to_vec();
    snapshot.into_iter().map(row_of).collect()
}

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
