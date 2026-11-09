//! Unix-socket listener for hook events.
//!
//! The helper binary connects, writes one JSON line, and exits. We never accept anything
//! from the network — the socket lives in the user's application-support directory with
//! owner-only permissions.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::thread;

use super::event::HookEvent;

pub fn socket_path(data_dir: &Path) -> PathBuf {
    data_dir.join("events.sock")
}

/// Bind the socket and forward parsed events to `tx`.
///
/// A stale socket file from a previous run is removed first; binding is the only operation
/// that can fail hard, and failing to listen must never take the app down.
pub fn spawn(data_dir: &Path, tx: Sender<HookEvent>) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(data_dir)?;
    let path = socket_path(data_dir);

    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }
    let listener = UnixListener::bind(&path)?;
    let _ = std::fs::set_permissions(
        &path,
        std::os::unix::fs::PermissionsExt::from_mode(0o600),
    );

    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() {
                continue;
            }
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(ev) = serde_json::from_str::<HookEvent>(line) {
                // Ack so the helper knows we took it. A dropped ack must not block the
                // agent, so the write result is intentionally ignored.
                let _ = reader.get_mut().write_all(b"ok\n");
                let _ = tx.send(ev);
            }
        }
    });

    Ok(path)
}
