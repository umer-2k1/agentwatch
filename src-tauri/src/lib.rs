//! AgentWatch — entry point.
//!
//! Wiring only: build the store, start the poll loop, expose two commands to the UI.
//! All logic lives in `providers/`, `resolver.rs`, `liveness.rs` and `store.rs`.

mod liveness;
mod model;
mod providers;
mod resolver;
mod store;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tauri::{Emitter, Manager};

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
            set_idle_after_s
        ])
        .run(tauri::generate_context!())
        .expect("error while running AgentWatch");
}
