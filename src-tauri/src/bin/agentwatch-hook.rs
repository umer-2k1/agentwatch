//! `agentwatch-hook` — the helper Claude Code runs at a lifecycle event.
//!
//! Fire-and-forget events: forward and exit.
//! `PreToolUse`: forward, then **wait** for the user's answer in the AgentWatch inbox, and
//! print a `permissionDecision` if one arrives in time.
//!
//! **It must never interfere with the user's agent.** No panics, no unbounded waits, no
//! noisy stdout, always exit 0. If anything at all goes wrong — AgentWatch closed, socket
//! gone, malformed payload, answer too slow — we print nothing, and Claude Code prompts in
//! the terminal exactly as it would have without us installed.

use std::io::Read;
use std::path::PathBuf;

fn main() {
    let _ = run();
}

fn run() -> Option<()> {
    let mut payload = String::new();
    std::io::stdin().read_to_string(&mut payload).ok()?;

    let ev = agentwatch_lib::hooks::forward::to_event(&payload)?;

    let decision = agentwatch_lib::hooks::forward::deliver(&socket_path(), &ev);
    if let Some(d) = decision {
        if let Some(out) = d.hook_output() {
            println!("{}", out);
        }
    }
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
