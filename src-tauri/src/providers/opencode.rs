//! OpenCode provider — reads `~/.local/share/opencode/opencode.db`.
//!
//! Verified against a live install on 2026-11-09. The modern schema is two tables:
//!
//! * `session_v2` — one row per session, with **explicit timing fields**:
//!   `time_busy` (a turn is running), `time_idle` (a turn finished), `time_archived`.
//! * `session_message` — the turns, `type` in {assistant, user, idle, system, …}, `seq`
//!   ordering, and a JSON `data` blob where an assistant message carries
//!   `time.completed` once it has finished streaming.
//!
//! OpenCode is unusual and welcome here: it records turn state as **data**, so we do not
//! have to infer it from prose the way we do for Claude Code. There is no hook, so nothing
//! can be answered — but `working` / `done` / `idle` are first-class facts.
//!
//! shortcut: we shell out to `sqlite3 -readonly` instead of linking a SQLite driver. The
//! app only ever reads, macOS always ships `sqlite3`, and it costs no build time or binary
//! size. Revisit if we ever need to write, or if the pragma/JSON behaviour differs on
//! Windows (M1.0).

use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

use super::{Provider, Verification};
use crate::model::SessionFact;

/// One row of the query. Field names come from the SQL aliases.
#[derive(Debug, Deserialize)]
struct Row {
    id: String,
    title: Option<String>,
    directory: Option<String>,
    time_updated: Option<i64>,
    time_busy: Option<i64>,
    time_idle: Option<i64>,
    #[serde(default)]
    last_type: Option<String>,
    #[serde(default)]
    last_completed: Option<i64>,
    #[serde(default)]
    last_tool: Option<String>,
}

pub struct OpenCodeProvider {
    db: PathBuf,
    /// Only sessions touched inside this window are candidates (D-012).
    window_s: i64,
}

impl OpenCodeProvider {
    pub fn new(window_hours: i64) -> Self {
        Self {
            db: db_path(),
            window_s: window_hours.max(1) * 3600,
        }
    }

    /// Run the query and return raw rows. Never panics: a locked or missing database simply
    /// yields no sessions this tick.
    fn rows(&self, now_ms: i64) -> Vec<Row> {
        if !self.db.exists() {
            return Vec::new();
        }
        let since = now_ms - self.window_s * 1000;

        // One query per tick for every live session, instead of one per session.
        let sql = format!(
            "SELECT s.id AS id,
                    s.title AS title,
                    s.directory AS directory,
                    s.time_updated AS time_updated,
                    s.time_busy AS time_busy,
                    s.time_idle AS time_idle,
                    (SELECT m.type FROM session_message m
                       WHERE m.session_id = s.id ORDER BY m.seq DESC LIMIT 1) AS last_type,
                    (SELECT json_extract(m.data, '$.time.completed') FROM session_message m
                       WHERE m.session_id = s.id AND m.type = 'assistant'
                       ORDER BY m.seq DESC LIMIT 1) AS last_completed,
                    (SELECT json_extract(m.data, '$.tool') FROM session_message m
                       WHERE m.session_id = s.id AND m.type = 'assistant'
                         AND json_extract(m.data, '$.time.completed') IS NULL
                       ORDER BY m.seq DESC LIMIT 1) AS last_tool
               FROM session_v2 s
              WHERE s.time_updated >= {since}
                AND s.time_archived IS NULL
                AND s.parent_id IS NULL
              ORDER BY s.time_updated DESC
              LIMIT 64;"
        );

        let out = Command::new("sqlite3")
            .args(["-readonly", "-json"])
            .arg(&self.db)
            .arg(&sql)
            .output();
        let Ok(out) = out else { return Vec::new() };
        if !out.status.success() || out.stdout.is_empty() {
            return Vec::new();
        }
        serde_json::from_slice::<Vec<Row>>(&out.stdout).unwrap_or_default()
    }
}

impl Provider for OpenCodeProvider {
    fn id(&self) -> &'static str {
        "opencode"
    }

    fn label(&self) -> &'static str {
        "OpenCode"
    }

    fn verification(&self) -> Verification {
        Verification::Verified
    }

    fn poll(&mut self, now_ms: i64) -> Vec<SessionFact> {
        self.rows(now_ms).into_iter().map(fact_from_row).collect()
    }
}

/// Pure mapping, so the status rules can be tested without a database.
fn fact_from_row(r: Row) -> SessionFact {
    let cwd = r.directory.clone().unwrap_or_default();
    let repo = std::path::Path::new(&cwd)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    // A turn is running when busy is set and is newer than the last idle mark.
    let busy = matches!((r.time_busy, r.time_idle), (Some(b), Some(i)) if b > i)
        || (r.time_busy.is_some() && r.time_idle.is_none());
    // An assistant message that has not completed is still streaming.
    let streaming = r.last_completed.is_none() && r.last_type.as_deref() == Some("assistant");
    let working = busy || streaming;

    SessionFact {
        id: format!("opencode:{}", r.id),
        provider: "opencode".into(),
        title: r.title.unwrap_or_else(|| repo.replace('-', " ")),
        cwd,
        repo,
        branch: String::new(), // OpenCode keeps no branch on the session row
        model: String::new(),
        turn_closed: !working,
        pending_tool: if working { r.last_tool.clone().or(Some("turn".into())) } else { None },
        last_activity_ms: r.time_updated.unwrap_or(0),
        turn_started_ms: r.time_busy,
        cost_usd: None,
        tokens_in: 0,
        tokens_out: 0,
        is_sidechain: false,
        readable: true,
    }
}

fn db_path() -> PathBuf {
    if let Ok(p) = std::env::var("AGENTWATCH_OPENCODE_DB") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".local/share/opencode/opencode.db")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(busy: Option<i64>, idle: Option<i64>, last: &str, completed: Option<i64>) -> Row {
        Row {
            id: "ses_x".into(),
            title: Some("fix the login test".into()),
            directory: Some("/Users/me/code/shop-api".into()),
            time_updated: Some(1_000_000),
            time_busy: busy,
            time_idle: idle,
            last_type: Some(last.into()),
            last_completed: completed,
            last_tool: None,
        }
    }

    #[test]
    fn busy_session_is_working() {
        let f = fact_from_row(row(Some(2_000), Some(1_000), "assistant", None));
        assert!(!f.turn_closed);
        assert_eq!(f.id, "opencode:ses_x");
        assert_eq!(f.repo, "shop-api");
    }

    #[test]
    fn idle_after_busy_is_closed() {
        let f = fact_from_row(row(Some(1_000), Some(2_000), "idle", Some(1_500)));
        assert!(f.turn_closed, "an idle mark after the busy mark closes the turn");
        assert!(f.pending_tool.is_none());
    }

    #[test]
    fn streaming_assistant_without_completion_is_working() {
        let f = fact_from_row(row(None, Some(1_000), "assistant", None));
        assert!(!f.turn_closed, "a message that never completed is still streaming");
    }

    #[test]
    fn completed_assistant_is_closed() {
        let f = fact_from_row(row(None, Some(1_000), "assistant", Some(900)));
        assert!(f.turn_closed);
    }

    #[test]
    fn title_falls_back_to_the_repo() {
        let mut r = row(None, Some(1), "idle", Some(1));
        r.title = None;
        r.directory = Some("/Users/me/code/my-cool-app".into());
        assert_eq!(fact_from_row(r).title, "my cool app");
    }

    #[test]
    fn missing_directory_does_not_panic() {
        let mut r = row(None, Some(1), "idle", Some(1));
        r.directory = None;
        let f = fact_from_row(r);
        assert!(f.cwd.is_empty());
        assert!(f.repo.is_empty());
    }
}
