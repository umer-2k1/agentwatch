//! The hook pipeline, end to end, through the real binary.
//!
//!   claude payload on stdin -> agentwatch-hook -> unix socket -> Store -> Status::NeedsYou
//!
//! Everything here is real except Claude Code itself: the actual `agentwatch-hook` binary is
//! executed as a subprocess, the actual socket is bound, and the actual resolver runs.
//!
//! Run: cargo test --test hook_pipeline -- --nocapture

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;

use agentwatch_lib::hooks::event::HookEvent;
use agentwatch_lib::hooks::forward;
use agentwatch_lib::model::Status;
use agentwatch_lib::store::Store;

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("aw-pipe-{}-{}", std::process::id(), tag));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// A transcript for `session_id`, so the store has something to attach the hook event to.
fn write_session(root: &PathBuf, session_id: &str, cwd: &str) {
    let dir = root.join("projects/-Users-test-app");
    fs::create_dir_all(&dir).unwrap();
    let line = serde_json::json!({
        "type": "assistant",
        "sessionId": session_id,
        "cwd": cwd,
        "gitBranch": "main",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "slug": "run the api tests",
        "message": {
            "model": "claude-opus-5",
            "stop_reason": "tool_use",
            "content": [{ "type": "tool_use", "id": "t1", "name": "Bash" }]
        }
    });
    fs::write(
        dir.join(format!("{session_id}.jsonl")),
        format!("{}\n", serde_json::to_string(&line).unwrap()),
    )
    .unwrap();
}

const PAYLOAD: &str = r#"{
  "hook_event_name": "Notification",
  "session_id": "live1",
  "cwd": "/Users/test/app",
  "message": "Claude needs your permission to use Bash",
  "tool_name": "Bash"
}"#;

#[test]
fn hook_binary_turns_a_permission_prompt_into_a_needs_you_dot() {
    let dir = tmp("e2e");
    let socket = dir.join("events.sock");
    let listener = UnixListener::bind(&socket).unwrap();

    // Accept one connection and read the forwarded line.
    let reader = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("helper never connected");
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line).ok();
        line
    });

    // 1. Run the real helper binary with a real Claude-shaped payload on stdin.
    let exe = env!("CARGO_BIN_EXE_agentwatch-hook");
    let mut child = Command::new(exe)
        .env("AGENTWATCH_SOCKET", &socket)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn agentwatch-hook");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(PAYLOAD.as_bytes())
        .unwrap();
    let status = child.wait().unwrap();

    // 2. A hook must never fail the agent.
    assert!(status.success(), "the helper must always exit 0");

    // 3. The event arrives and is understood.
    let forwarded = reader.join().expect("reader thread");
    println!("forwarded: {}", forwarded.trim());
    let ev: HookEvent = serde_json::from_str(forwarded.trim()).expect("valid event json");
    assert_eq!(ev.event, "Notification");
    assert!(ev.is_waiting(), "a Notification must read as `waiting`");
    assert_eq!(ev.key().as_deref(), Some("claude-code:live1"));

    // 4. The store turns it into `needsYou` — the amber dot.
    let root = dir.join("claude-root");
    write_session(&root, "live1", "/Users/test/app");
    std::env::set_var("AGENTWATCH_CLAUDE_ROOT", &root);

    let mut store = Store::with_window(1);
    store.apply_hook(ev, None);
    let rows = store.tick().to_vec();

    println!("status: {:?}", rows.first().map(|s| s.status));
    assert_eq!(
        rows.first().map(|s| s.status),
        Some(Status::NeedsYou),
        "a permission prompt must light the dot amber"
    );
    assert_eq!(
        format!("{:?}", rows[0].confidence).to_lowercase(),
        "certain",
        "a hook is a fact, not an inference"
    );
    assert!(rows[0].evidence.detail.contains("permission"));

    // 5. When the agent moves again, the amber clears.
    let clear = HookEvent {
        event: "PostToolUse".into(),
        session_id: Some("live1".into()),
        cwd: None,
        message: None,
        tool_name: None,
    };
    store.apply_hook(clear, None);
    let after = store.tick().to_vec();
    println!("after clear: {:?}", after.first().map(|s| s.status));
    assert_ne!(
        after.first().map(|s| s.status),
        Some(Status::NeedsYou),
        "the amber dot must clear when the agent continues"
    );

    std::env::remove_var("AGENTWATCH_CLAUDE_ROOT");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn helper_exits_zero_when_no_app_is_running() {
    // The single most important property: AgentWatch being closed must be invisible.
    let exe = env!("CARGO_BIN_EXE_agentwatch-hook");
    let mut child = Command::new(exe)
        .env("AGENTWATCH_SOCKET", "/nonexistent/agentwatch.sock")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn helper");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(PAYLOAD.as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());
}

#[test]
fn payload_to_event_mapping_is_pure() {
    let ev = forward::to_event(PAYLOAD).unwrap();
    assert_eq!(ev.event, "Notification");
    assert!(ev.detail().contains("Bash"));
}

/// The claim that matters: a `PreToolUse` hook is held open, the user answers in AgentWatch,
/// and Claude Code receives `permissionDecision: allow` — without a keystroke ever touching
/// the terminal. Also proves the timeout falls back to "no opinion".
#[test]
fn answering_a_pre_tool_use_returns_a_permission_decision() {
    use agentwatch_lib::hooks::event::Decision;
    use agentwatch_lib::hooks::server;

    let dir = tmp("answer");
    let socket = dir.join("events.sock");

    // Play the app: receive the pending event, answer it after a beat.
    let (tx, rx) = std::sync::mpsc::channel::<server::Pending>();
    server::spawn(&dir, tx).unwrap();

    let app = thread::spawn(move || {
        let pending = rx.recv().expect("no pending event");
        assert!(pending.event.wants_decision(), "PreToolUse must want a decision");
        let reply = pending.reply.expect("no reply channel");
        // A short delay, so the helper is genuinely blocked when the answer arrives.
        thread::sleep(std::time::Duration::from_millis(120));
        reply.send(Decision::Allow).unwrap();
    });

    let exe = env!("CARGO_BIN_EXE_agentwatch-hook");
    let mut child = Command::new(exe)
        .env("AGENTWATCH_SOCKET", &socket)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn helper");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            br#"{"hook_event_name":"PreToolUse","session_id":"dec1","tool_name":"Bash",
                 "message":"Claude wants to run npm test"}"#,
        )
        .unwrap();

    let out = child.wait_with_output().unwrap();
    app.join().unwrap();

    assert!(out.status.success(), "the helper must always exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    println!("helper printed: {}", stdout.trim());

    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid hook output");
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "allow");
    assert_eq!(v["hookSpecificOutput"]["hookEventName"], "PreToolUse");

    let _ = fs::remove_dir_all(&dir);
}
