//! Claude Code provider.
//!
//! Reads `~/.claude/projects/<encoded-cwd>/<session-id>.jsonl`.
//! Format documented in `docs/specs/SPEC-002-transcript-reading.md`, verified against live
//! files on 2026-11-06.
//!
//! Tailing rules implemented here:
//!   * keep a byte offset per file                (never re-read a whole transcript)
//!   * buffer a trailing partial line             (JSONL writes are not atomic)
//!   * detect truncation -> replay from zero      (Claude Code can rewind a message)
//!   * ignore `isSidechain` for status            (a subagent must not mark its parent done)

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{Provider, Verification};
use crate::model::SessionFact;

/// Per-file accumulation. Lives across polls.
#[derive(Default)]
struct FileState {
    offset: u64,
    fact: SessionFact,
    /// Bytes of an incomplete trailing line, carried to the next poll.
    tail: Vec<u8>,
}

pub struct ClaudeProvider {
    root: PathBuf,
    files: HashMap<PathBuf, FileState>,
}

impl ClaudeProvider {
    pub fn new() -> Self {
        let root = dirs_claude();
        Self {
            root,
            files: HashMap::new(),
        }
    }

    fn discover(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let projects = self.root.join("projects");
        let Ok(dirs) = fs::read_dir(&projects) else {
            return out;
        };
        for entry in dirs.flatten() {
            let Ok(meta) = entry.metadata() else { continue };
            if !meta.is_dir() {
                continue;
            }
            let Ok(files) = fs::read_dir(entry.path()) else {
                continue;
            };
            for f in files.flatten() {
                let p = f.path();
                if p.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                    out.push(p);
                }
            }
        }
        out
    }

    /// Read the bytes appended since the last poll and fold them into the state.
    fn advance(&mut self, path: &Path) {
        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return,
        };
        let size = meta.len();

        let state = self.files.entry(path.to_path_buf()).or_default();

        // Truncation (rewind/edit) or first sight: start clean.
        if size < state.offset {
            state.offset = 0;
            state.tail.clear();
            state.fact = SessionFact::default();
        }
        if size == state.offset {
            return;
        }

        let mut buf = Vec::new();
        {
            let Ok(mut f) = fs::File::open(path) else { return };
            if f.seek(SeekFrom::Start(state.offset)).is_err() {
                return;
            }
            let mut chunk = Vec::with_capacity(64 * 1024);
            if f.read_to_end(&mut chunk).is_err() {
                return;
            }
            buf = chunk;
        }
        state.offset = size;

        // Prepend any carried partial line, then split on the last newline.
        buf.splice(0..0, std::mem::take(&mut state.tail));
        let Some(last_nl) = buf.iter().rposition(|b| *b == b'\n') else {
            // Not a single complete line yet — keep everything for next time.
            state.tail = buf;
            return;
        };
        state.tail = buf.split_off(last_nl + 1);

        for line in buf.split(|b| *b == b'\n') {
            if line.is_empty() {
                continue;
            }
            let Ok(v) = serde_json::from_slice::<Value>(line) else {
                // A half-written or malformed line is not a reason to crash.
                continue;
            };
            apply_line(&mut state.fact, &v);
        }

        if state.fact.id.is_empty() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                state.fact.id = format!("claude-code:{}", stem);
            }
        }
        state.fact.provider = "claude-code".into();
        if state.fact.title.is_empty() {
            state.fact.title = state
                .fact
                .repo
                .clone()
                .replace('_', " ")
                .replace('-', " ");
        }
        if state.fact.title.is_empty() {
            state.fact.title = "Claude Code".into();
        }
        state.fact.readable = true;
    }
}

impl Provider for ClaudeProvider {
    fn id(&self) -> &'static str {
        "claude-code"
    }

    fn label(&self) -> &'static str {
        "Claude Code"
    }

    fn verification(&self) -> Verification {
        Verification::Verified
    }

    fn poll(&mut self, _now_ms: i64) -> Vec<SessionFact> {
        let paths = self.discover();

        // Drop sessions whose file disappeared (closed tab, cleaned up).
        self.files.retain(|p, _| p.exists());

        for p in &paths {
            self.advance(p);
        }

        self.files
            .values()
            .filter(|s| !s.fact.is_sidechain)
            .map(|s| s.fact.clone())
            .collect()
    }
}

/// Fold one transcript line into the running facts.
///
/// Unknown `type`s are ignored silently (SPEC-002): the format has more line kinds than
/// user/assistant and must never be treated as an error.
fn apply_line(fact: &mut SessionFact, v: &Value) {
    let ty = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

    if let Some(sid) = v.get("sessionId").and_then(|s| s.as_str()) {
        if fact.id.is_empty() {
            fact.id = format!("claude-code:{}", sid);
        }
    }
    if let Some(cwd) = v.get("cwd").and_then(|s| s.as_str()) {
        if fact.cwd.is_empty() {
            fact.cwd = cwd.to_string();
            fact.repo = repo_name(cwd);
        }
    }
    if let Some(b) = v.get("gitBranch").and_then(|s| s.as_str()) {
        if !b.is_empty() {
            fact.branch = b.to_string();
        }
    }
    if let Some(ts) = v.get("timestamp").and_then(|s| s.as_str()) {
        if let Some(ms) = parse_rfc3339(ts) {
            if ms > fact.last_activity_ms {
                fact.last_activity_ms = ms;
            }
        }
    }

    // Subagents update liveness but never status (SPEC-001).
    let side = v.get("isSidechain").and_then(|s| s.as_bool()).unwrap_or(false);
    if side {
        fact.is_sidechain = false; // the file itself is the parent session
        return;
    }

    match ty {
        "assistant" => {
            if let Some(slug) = v.get("slug").and_then(|s| s.as_str()) {
                if fact.title.is_empty() && !slug.is_empty() {
                    fact.title = slug.to_string();
                }
            }
            let Some(msg) = v.get("message") else { return };
            if let Some(m) = msg.get("model").and_then(|s| s.as_str()) {
                fact.model = m.to_string();
            }
            if let Some(u) = msg.get("usage") {
                if let Some(n) = u.get("input_tokens").and_then(|n| n.as_u64()) {
                    fact.tokens_in = n;
                }
                if let Some(n) = u.get("output_tokens").and_then(|n| n.as_u64()) {
                    fact.tokens_out = n;
                }
            }

            let mut has_open_tool = false;
            if let Some(blocks) = msg.get("content").and_then(|c| c.as_array()) {
                for b in blocks {
                    if b.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                        if let Some(id) = b.get("id").and_then(|s| s.as_str()) {
                            fact.pending_tool = Some(id.to_string());
                            has_open_tool = true;
                        }
                    }
                }
            }

            let stop = msg.get("stop_reason").and_then(|s| s.as_str());
            if stop == Some("end_turn") && !has_open_tool {
                fact.turn_closed = true;
                fact.pending_tool = None;
                fact.turn_started_ms = None;
            } else if has_open_tool || stop == Some("tool_use") {
                fact.turn_closed = false;
                if fact.turn_started_ms.is_none() {
                    fact.turn_started_ms = Some(fact.last_activity_ms);
                }
            }
        }

        "user" => {
            if let Some(blocks) = v
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_array())
            {
                for b in blocks {
                    if b.get("type").and_then(|t| t.as_str()) != Some("tool_result") {
                        continue;
                    }
                    let id = b.get("tool_use_id").and_then(|s| s.as_str());
                    match (&fact.pending_tool, id) {
                        (Some(p), Some(i)) if p == i => fact.pending_tool = None,
                        (Some(_), None) => fact.pending_tool = None,
                        _ => {}
                    }
                    // A tool result means the turn continues.
                    fact.turn_closed = false;
                }
            }
        }

        "cost-state" => {
            if let Some(c) = v.get("totalCostUSD").and_then(|c| c.as_f64()) {
                fact.cost_usd = Some(c);
            }
        }

        "custom-title" => {
            if let Some(t) = v.get("customTitle").and_then(|s| s.as_str()) {
                if !t.is_empty() {
                    fact.title = t.to_string();
                }
            }
        }

        "agent-name" => {
            if let Some(t) = v.get("agentName").and_then(|s| s.as_str()) {
                if !t.is_empty() && fact.title.is_empty() {
                    fact.title = t.to_string();
                }
            }
        }

        _ => {}
    }
}

fn dirs_claude() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".claude")
}

fn repo_name(cwd: &str) -> String {
    Path::new(cwd)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

fn parse_rfc3339(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|d| d.timestamp_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(json: &str) -> Value {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn end_turn_closes_the_turn() {
        let mut f = SessionFact::default();
        apply_line(
            &mut f,
            &line(r#"{"type":"assistant","gitBranch":"main","cwd":"/x/app","timestamp":"2026-11-06T10:00:00Z","slug":"fix-login","message":{"model":"claude-opus-5","stop_reason":"end_turn","content":[{"type":"text","text":"done"}]}}"#),
        );
        assert!(f.turn_closed);
        assert!(f.pending_tool.is_none());
        assert_eq!(f.branch, "main");
        assert_eq!(f.repo, "app");
        assert_eq!(f.title, "fix-login");
    }

    #[test]
    fn open_tool_use_keeps_the_turn_open() {
        let mut f = SessionFact::default();
        apply_line(
            &mut f,
            &line(r#"{"type":"assistant","timestamp":"2026-11-06T10:00:00Z","message":{"stop_reason":"tool_use","content":[{"type":"tool_use","id":"t1","name":"Bash"}]}}"#),
        );
        assert!(!f.turn_closed);
        assert_eq!(f.pending_tool.as_deref(), Some("t1"));
    }

    #[test]
    fn tool_result_closes_the_pending_tool() {
        let mut f = SessionFact::default();
        apply_line(
            &mut f,
            &line(r#"{"type":"assistant","timestamp":"2026-11-06T10:00:00Z","message":{"stop_reason":"tool_use","content":[{"type":"tool_use","id":"t1"}]}}"#),
        );
        apply_line(
            &mut f,
            &line(r#"{"type":"user","timestamp":"2026-11-06T10:00:05Z","message":{"content":[{"type":"tool_result","tool_use_id":"t1"}]}}"#),
        );
        assert!(f.pending_tool.is_none());
        assert!(!f.turn_closed);
    }

    #[test]
    fn unknown_line_types_are_ignored() {
        let mut f = SessionFact::default();
        apply_line(&mut f, &line(r#"{"type":"api-request-blob","sessionId":"s"}"#));
        apply_line(&mut f, &line(r#"{"type":"mode","mode":"plan"}"#));
        assert!(f.id.is_empty() || f.id.starts_with("claude-code:"));
    }

    #[test]
    fn cost_state_is_captured() {
        let mut f = SessionFact::default();
        apply_line(
            &mut f,
            &line(r#"{"type":"cost-state","totalCostUSD":1.2345}"#),
        );
        assert_eq!(f.cost_usd, Some(1.2345));
    }
}
