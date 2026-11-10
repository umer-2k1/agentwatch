//! Codex provider — reads `~/.codex/sessions/**/rollout-*.jsonl`.
//!
//! Verified against real rollouts on 2026-11-09. Each line is
//! `{ type, timestamp, ordinal, payload }`, where `payload.type` carries the meaning:
//!
//! | record | payload.type | meaning |
//! |---|---|---|
//! | `session_meta` | — | session id, cwd, source (cli / exec / app) |
//! | `event_msg` | `task_started` | a turn began |
//! | `event_msg` | `task_complete` | the turn ended — includes `last_agent_message` |
//! | `event_msg` | `token_count` | usage for the turn |
//! | `response_item` | `custom_tool_call` | a tool call started |
//! | `response_item` | `custom_tool_call_output` | that call returned |
//!
//! Codex does **not** record permission prompts in the rollout, so `needsYou` comes from
//! the hook (`~/.codex/hooks.json`) the same way it does for Claude Code.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{Provider, Verification};
use crate::model::SessionFact;

pub struct CodexProvider {
    root: PathBuf,
    window_s: i64,
    files: HashMap<PathBuf, FileState>,
}

#[derive(Default)]
struct FileState {
    offset: u64,
    fact: SessionFact,
}

impl CodexProvider {
    pub fn new(window_hours: i64) -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        Self {
            root: PathBuf::from(home).join(".codex/sessions"),
            window_s: window_hours.max(1) * 3600,
            files: HashMap::new(),
        }
    }

    fn discover(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let cutoff =
            std::time::SystemTime::now() - std::time::Duration::from_secs(self.window_s as u64);
        walk(&self.root, cutoff, &mut out, 0);
        out
    }

    fn advance(&mut self, path: &Path) {
        let Ok(meta) = std::fs::metadata(path) else {
            return;
        };
        let size = meta.len();
        let state = self.files.entry(path.to_path_buf()).or_default();

        if size < state.offset {
            state.offset = 0;
            state.fact = SessionFact::default();
        }
        if size == state.offset {
            return;
        }

        use std::io::{Read, Seek, SeekFrom};
        let Ok(mut f) = std::fs::File::open(path) else {
            return;
        };
        if f.seek(SeekFrom::Start(state.offset)).is_err() {
            return;
        }
        let mut buf = Vec::new();
        if f.read_to_end(&mut buf).is_err() {
            return;
        }
        state.offset = size;

        // Only complete lines; JSONL writes are not atomic.
        let Some(last_nl) = buf.iter().rposition(|b| *b == b'\n') else {
            state.offset = size - buf.len() as u64; // re-read next time
            return;
        };
        let complete = &buf[..=last_nl];
        // Put the partial tail back for the next poll.
        state.offset = size - (buf.len() - complete.len()) as u64;

        for line in complete.split(|b| *b == b'\n') {
            if line.is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_slice::<Value>(line) {
                apply(&mut state.fact, &v);
            }
        }

        state.fact.provider = "codex".into();
        if state.fact.id.is_empty() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                state.fact.id = format!("codex:{stem}");
            }
        }
        if state.fact.title.is_empty() {
            state.fact.title = state.fact.repo.replace('-', " ");
        }
        if state.fact.title.is_empty() {
            state.fact.title = "Codex".into();
        }
        state.fact.readable = true;
    }
}

impl Provider for CodexProvider {
    fn id(&self) -> &'static str {
        "codex"
    }
    fn label(&self) -> &'static str {
        "Codex"
    }
    fn verification(&self) -> Verification {
        Verification::Verified
    }

    fn poll(&mut self, _now_ms: i64) -> Vec<SessionFact> {
        let paths = self.discover();
        self.files.retain(|p, _| p.exists());
        for p in paths {
            self.advance(&p);
        }
        self.files
            .values()
            .map(|s| s.fact.clone())
            .filter(|f| !f.id.is_empty())
            .collect()
    }
}

fn walk(dir: &Path, cutoff: std::time::SystemTime, out: &mut Vec<PathBuf>, depth: u32) {
    if depth > 5 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        let Ok(md) = e.metadata() else { continue };
        if md.is_dir() {
            walk(&p, cutoff, out, depth + 1);
        } else if p.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            if md.modified().map(|m| m >= cutoff).unwrap_or(false) {
                out.push(p);
            }
        }
    }
}

fn apply(fact: &mut SessionFact, v: &Value) {
    let ty = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let payload = v.get("payload").unwrap_or(&Value::Null);
    let pty = payload.get("type").and_then(|t| t.as_str()).unwrap_or("");

    if let Some(ts) = v.get("timestamp").and_then(|t| t.as_str()) {
        if let Some(ms) = chrono::DateTime::parse_from_rfc3339(ts)
            .ok()
            .map(|d| d.timestamp_millis())
        {
            if ms > fact.last_activity_ms {
                fact.last_activity_ms = ms;
            }
        }
    }

    match ty {
        "session_meta" => {
            if let Some(id) = payload.get("session_id").and_then(|s| s.as_str()) {
                fact.id = format!("codex:{id}");
            }
            if let Some(cwd) = payload.get("cwd").and_then(|s| s.as_str()) {
                fact.cwd = cwd.to_string();
                fact.repo = Path::new(cwd)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
            }
        }
        "event_msg" => match pty {
            "task_started" => {
                fact.turn_closed = false;
                fact.pending_tool = Some("turn".into());
                fact.turn_started_ms = Some(fact.last_activity_ms);
            }
            "task_complete" => {
                fact.turn_closed = true;
                fact.pending_tool = None;
                fact.turn_started_ms = None;
            }
            "token_count" => {
                if let Some(u) = payload.get("info").and_then(|i| i.get("total_token_usage")) {
                    if let Some(n) = u.get("input_tokens").and_then(|n| n.as_u64()) {
                        fact.tokens_in = n;
                    }
                    if let Some(n) = u.get("output_tokens").and_then(|n| n.as_u64()) {
                        fact.tokens_out = n;
                    }
                }
            }
            _ => {}
        },
        "response_item" => match pty {
            "custom_tool_call" => {
                fact.pending_tool = Some(
                    payload
                        .get("name")
                        .and_then(|s| s.as_str())
                        .unwrap_or("tool")
                        .to_string(),
                );
                fact.turn_closed = false;
            }
            "custom_tool_call_output" => {
                fact.pending_tool = None;
            }
            "message" => {
                if payload.get("role").and_then(|r| r.as_str()) == Some("assistant") {
                    if let Some(text) = first_text(payload) {
                        // The first line of the last assistant message makes a good title.
                        if let Some(line) = text.lines().find(|l| !l.trim().is_empty()) {
                            let line = line.trim().trim_start_matches('#').trim();
                            if !line.is_empty() && line.len() < 80 {
                                fact.title = line.to_string();
                            }
                        }
                    }
                }
            }
            _ => {}
        },
        _ => {}
    }
}

fn first_text(payload: &Value) -> Option<String> {
    let content = payload.get("content")?.as_array()?;
    for c in content {
        if let Some(t) = c.get("text").and_then(|t| t.as_str()) {
            return Some(t.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Value {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn session_meta_sets_identity_and_repo() {
        let mut f = SessionFact::default();
        apply(
            &mut f,
            &v(
                r#"{"type":"session_meta","payload":{"session_id":"01abc","cwd":"/Users/me/code/shop-api"}}"#,
            ),
        );
        assert_eq!(f.id, "codex:01abc");
        assert_eq!(f.repo, "shop-api");
    }

    #[test]
    fn task_started_opens_a_turn() {
        let mut f = SessionFact::default();
        apply(
            &mut f,
            &v(
                r#"{"type":"event_msg","timestamp":"2026-11-09T10:00:00Z","payload":{"type":"task_started"}}"#,
            ),
        );
        assert!(!f.turn_closed);
        assert_eq!(f.pending_tool.as_deref(), Some("turn"));
        assert!(f.turn_started_ms.is_some());
    }

    #[test]
    fn task_complete_closes_it() {
        let mut f = SessionFact::default();
        apply(
            &mut f,
            &v(r#"{"type":"event_msg","payload":{"type":"task_started"}}"#),
        );
        apply(
            &mut f,
            &v(r#"{"type":"event_msg","payload":{"type":"task_complete"}}"#),
        );
        assert!(f.turn_closed);
        assert!(f.pending_tool.is_none());
        assert!(f.turn_started_ms.is_none());
    }

    #[test]
    fn tool_call_and_output_track_pending() {
        let mut f = SessionFact::default();
        apply(
            &mut f,
            &v(r#"{"type":"response_item","payload":{"type":"custom_tool_call","name":"shell"}}"#),
        );
        assert_eq!(f.pending_tool.as_deref(), Some("shell"));
        apply(
            &mut f,
            &v(r#"{"type":"response_item","payload":{"type":"custom_tool_call_output"}}"#),
        );
        assert!(f.pending_tool.is_none());
    }

    #[test]
    fn token_count_is_recorded() {
        let mut f = SessionFact::default();
        apply(
            &mut f,
            &v(
                r#"{"type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":100,"output_tokens":20}}}}"#,
            ),
        );
        assert_eq!(f.tokens_in, 100);
        assert_eq!(f.tokens_out, 20);
    }

    #[test]
    fn assistant_message_becomes_a_title() {
        let mut f = SessionFact::default();
        apply(
            &mut f,
            &v(
                r#"{"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Fixed the flaky login test"}]}}"#,
            ),
        );
        assert_eq!(f.title, "Fixed the flaky login test");
    }

    #[test]
    fn unknown_records_are_ignored() {
        let mut f = SessionFact::default();
        apply(&mut f, &v(r#"{"type":"world_state","payload":{}}"#));
        apply(&mut f, &v(r#"{"type":"nonsense"}"#));
        assert!(f.id.is_empty());
    }
}
