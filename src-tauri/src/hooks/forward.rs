//! Turning Claude Code's hook payload into our event, and delivering it.
//!
//! This is the only place that knows the vendor's payload shape. Keep it that way: when
//! Claude Code changes a field name, exactly one function breaks and exactly one test fails.
//!
//! Two rules govern everything here:
//!
//! 1. **Never break the agent.** The hook runs inside the user's session. A panic, a panic
//!    on a missing field, or a hang would block real work. Every failure path returns
//!    quietly.
//! 2. **Never block for long.** A hook that waits holds the agent. A 400 ms timeout is the
//!    difference between "invisible" and "why is Claude slow?".

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use super::event::HookEvent;

/// How long the helper will wait for the app to acknowledge. Deliberately tiny.
pub const ACK_TIMEOUT: Duration = Duration::from_millis(400);

/// Normalise a raw payload. Returns `None` when there is nothing useful in it.
pub fn to_event(payload: &str) -> Option<HookEvent> {
    let v: serde_json::Value = serde_json::from_str(payload).ok()?;

    let get = |keys: &[&str]| -> Option<String> {
        keys.iter()
            .find_map(|k| v.get(*k).and_then(|x| x.as_str()))
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
    };

    // Claude Code sends `hook_event_name`; accept `event` too so the helper can be tested
    // and driven by hand.
    let event = get(&["hook_event_name", "event", "type"])?;

    Some(HookEvent {
        event,
        session_id: get(&["session_id", "sessionId"]),
        cwd: get(&["cwd", "working_directory"]),
        message: get(&["message", "reason", "notification", "title"]),
        tool_name: get(&["tool_name", "toolName"]),
    })
}

/// Deliver an event. Best-effort by design: a missing app is not an error worth reporting
/// to the user's agent, so the result is a plain bool rather than a `Result`.
pub fn send(socket: &std::path::Path, ev: &HookEvent) -> bool {
    let Ok(mut stream) = UnixStream::connect(socket) else {
        return false;
    };
    let _ = stream.set_write_timeout(Some(ACK_TIMEOUT));
    let _ = stream.set_read_timeout(Some(ACK_TIMEOUT));

    let Ok(mut line) = serde_json::to_string(ev) else {
        return false;
    };
    line.push('\n');

    if stream.write_all(line.as_bytes()).is_err() {
        return false;
    }
    // Read the ack, but never insist on it.
    let mut ack = String::new();
    let _ = BufReader::new(stream).read_line(&mut ack);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::os::unix::net::UnixListener;

    #[test]
    fn reads_claude_payload() {
        let ev = to_event(
            r#"{"hook_event_name":"Notification","session_id":"s1","cwd":"/x/app",
                "message":"Claude needs your permission to use Bash","tool_name":"Bash"}"#,
        )
        .unwrap();
        assert_eq!(ev.event, "Notification");
        assert_eq!(ev.key().as_deref(), Some("claude-code:s1"));
        assert!(ev.is_waiting());
        assert!(ev.detail().contains("permission"));
        assert!(ev.detail().contains("Bash"));
    }

    #[test]
    fn tolerates_missing_optional_fields() {
        let ev = to_event(r#"{"hook_event_name":"Stop","cwd":"/x"}"#).unwrap();
        assert_eq!(ev.event, "Stop");
        assert!(ev.session_id.is_none());
        assert!(ev.clears_waiting());
    }

    #[test]
    fn ignores_garbage_without_panicking() {
        assert!(to_event("not json").is_none());
        assert!(to_event("{}").is_none());
        assert!(to_event("").is_none());
        assert!(to_event("null").is_none());
    }

    #[test]
    fn empty_problem_fields_are_treated_as_absent() {
        let ev = to_event(r#"{"hook_event_name":"Stop","session_id":"","message":""}"#).unwrap();
        assert!(ev.session_id.is_none());
        assert_eq!(ev.detail(), "waiting for you");
    }

    #[test]
    fn delivers_over_a_real_socket() {
        let dir = std::env::temp_dir().join(format!("aw-fwd-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.sock");
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();

        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = String::new();
            stream.read_to_string(&mut buf).unwrap();
            buf
        });

        let ev = to_event(r#"{"hook_event_name":"Notification","session_id":"s9"}"#).unwrap();
        assert!(send(&path, &ev));

        let got = handle.join().unwrap();
        assert!(got.contains("\"Notification\""));
        assert!(got.contains("s9"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_socket_is_not_an_error() {
        let ev = to_event(r#"{"hook_event_name":"Stop"}"#).unwrap();
        assert!(!send(std::path::Path::new("/nonexistent/aw.sock"), &ev));
    }
}
