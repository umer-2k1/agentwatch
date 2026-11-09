//! `agentwatch-hook` — the helper Claude Code runs at a lifecycle event.
//!
//! Claude Code writes a JSON payload to stdin. We normalise it and forward it to the
//! AgentWatch app over a unix socket, then exit.
//!
//! **It must never interfere with the user's agent.** No panics, no hangs, no noisy stdout,
//! always exit 0. If AgentWatch is not running, this is a no-op that costs a few hundred
//! microseconds.

use std::io::Read;
use std::path::PathBuf;

fn main() {
    // Exit 0 no matter what. A hook that fails our way must not fail the agent's way.
    let _ = run();
}

fn run() -> Option<()> {
    let mut payload = String::new();
    std::io::stdin().read_to_string(&mut payload).ok()?;

    let ev = agentwatch_lib::hooks::forward::to_event(&payload)?;
    agentwatch_lib::hooks::forward::send(&socket_path(), &ev);

    // v0.2 answers here: print a `hookSpecificOutput` decision instead of nothing.
    None
}

fn socket_path() -> PathBuf {
    if let Ok(p) = std::env::var("AGENTWATCH_SOCKET") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Library/Application Support/AgentWatch/events.sock")
}
