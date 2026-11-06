//! Core data types shared by every layer.
//!
//! Dependency rule (docs/architecture/ARCHITECTURE.md): this module knows nothing about
//! Tauri, the filesystem, or any specific agent.

use serde::{Deserialize, Serialize};

/// What an agent is doing right now. See SPEC-001.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Working,
    NeedsYou,
    Done,
    Idle,
    /// Process is gone. The state competitors cannot produce (D-009).
    Dead,
    /// We genuinely do not know. Never silently rendered as Idle.
    Unknown,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Working => "working",
            Status::NeedsYou => "needsYou",
            Status::Done => "done",
            Status::Idle => "idle",
            Status::Dead => "dead",
            Status::Unknown => "unknown",
        }
    }

    /// Sort weight for the strip and the inbox: most urgent first.
    pub fn urgency(&self) -> u8 {
        match self {
            Status::NeedsYou => 0,
            Status::Dead => 1,
            Status::Working => 2,
            Status::Done => 3,
            Status::Idle => 4,
            Status::Unknown => 5,
        }
    }
}

/// How much we trust a status. Rendered in the UI, not hidden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Confidence {
    /// From a hook or from process liveness — a fact.
    Certain,
    /// From transcript content — a trace.
    Inferred,
    /// Unreadable, or no liveness data.
    Unknown,
}

/// Proof for a status, so the UI can answer "why does it say that?".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub kind: EvidenceKind,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceKind {
    Transcript,
    Hook,
    Process,
}

/// Everything a provider can learn about one session by reading its files.
///
/// This is deliberately *facts only* — no status. Deciding status is the resolver's job,
/// which keeps the resolver unit-testable with zero I/O (ADR-002).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFact {
    pub id: String,
    pub provider: String,
    pub title: String,
    pub cwd: String,
    pub repo: String,
    pub branch: String,
    pub model: String,

    /// Last `assistant` line ended with stop_reason == "end_turn".
    pub turn_closed: bool,
    /// A `tool_use` block is still open (no matching `tool_result`).
    pub pending_tool: Option<String>,
    /// Epoch millis of the last parsable transcript line.
    pub last_activity_ms: i64,
    /// Epoch millis when the current turn started.
    pub turn_started_ms: Option<i64>,

    /// From the agent's own cost record when it has one; else None.
    pub cost_usd: Option<f64>,
    pub tokens_in: u64,
    pub tokens_out: u64,

    /// True when this session is a subagent, not a top-level session.
    pub is_sidechain: bool,
    /// The transcript was parseable at all.
    pub readable: bool,
}

/// One row of the strip / inbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    #[serde(flatten)]
    pub fact: SessionFact,

    pub status: Status,
    pub confidence: Confidence,
    pub evidence: Evidence,

    /// Pid of the agent process, when known.
    pub pid: Option<i32>,
    /// How long since the last activity, in seconds.
    pub quiet_for_s: i64,
    /// How long the current turn has been running, in seconds.
    pub turn_age_s: Option<i64>,
}
