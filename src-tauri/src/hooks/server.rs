//! Unix-socket listener for hook events, and the trip back.
//!
//! Two kinds of traffic:
//!
//! * **Fire-and-forget** (`Notification`, `Stop`, …) — the helper writes a line and leaves.
//! * **Decision** (`PreToolUse`) — the helper writes a line and *waits*. We hold the
//!   connection open until the user answers in the inbox, then write the decision back.
//!
//! The wait is bounded on both sides. A hook that never returns would freeze the user's
//! agent, which is a far worse failure than "I did not answer in time".

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

use super::event::{Decision, HookEvent};

/// How long the app will hold a `PreToolUse` connection waiting for a human.
/// The helper times out slightly earlier, so it is always the helper that gives up first.
pub const DECISION_TIMEOUT: Duration = Duration::from_millis(15_000);

/// An event plus, for decision-capable events, the channel to answer on.
pub struct Pending {
    pub event: HookEvent,
    pub reply: Option<Sender<Decision>>,
}

pub fn socket_path(data_dir: &Path) -> PathBuf {
    data_dir.join("events.sock")
}

pub fn spawn(data_dir: &Path, tx: Sender<Pending>) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(data_dir)?;
    let path = socket_path(data_dir);

    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }
    let listener = UnixListener::bind(&path)?;
    let _ = std::fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o600));

    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            // One thread per connection: a waiting PreToolUse must not block the next
            // Notification from another agent.
            let tx = tx.clone();
            thread::spawn(move || handle(stream, tx));
        }
    });

    Ok(path)
}

fn handle(mut stream: std::os::unix::net::UnixStream, tx: Sender<Pending>) {
    let mut line = String::new();
    {
        let mut reader = BufReader::new(&stream);
        if reader.read_line(&mut line).is_err() {
            return;
        }
    }
    let line = line.trim();
    if line.is_empty() {
        return;
    }
    let Ok(ev) = serde_json::from_str::<HookEvent>(line) else {
        return;
    };

    if !ev.wants_decision() {
        let _ = stream.write_all(b"ok\n");
        let _ = tx.send(Pending { event: ev, reply: None });
        return;
    }

    let (reply_tx, reply_rx): (Sender<Decision>, Receiver<Decision>) = std::sync::mpsc::channel();
    if tx
        .send(Pending {
            event: ev,
            reply: Some(reply_tx),
        })
        .is_err()
    {
        return;
    }

    // Nobody answers => `Ask`, which hands the question back to Claude Code's own prompt.
    let decision = reply_rx.recv_timeout(DECISION_TIMEOUT).unwrap_or(Decision::Ask);
    let body = serde_json::json!({ "decision": decision });
    let mut out = body.to_string();
    out.push('\n');
    let _ = stream.write_all(out.as_bytes());
}
