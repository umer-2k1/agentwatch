//! Core data types shared by every layer.
//!
//! Covers Blueprint §10 (F1–F13), §13 (Data models), §14 (UI/UX), and §19 (MCP/iPhone schemas).
//! Dependency rule: this module knows nothing about Tauri, the filesystem, or any specific agent.

use serde::{Deserialize, Serialize};

/// What an agent is doing right now. See SPEC-001 and Blueprint §10.F3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Working,
    NeedsYou,
    Done,
    Idle,
    /// Process is gone or turn failed with an error (auth/balance/crash).
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

    /// Sort weight for the strip, hover card and inbox: most urgent first.
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
    Certain,
    Inferred,
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

/// A file modified by an agent tool call (`Edit`, `MultiEdit`, `Write`, `NotebookEdit`, `apply_patch`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEdit {
    pub path: String,
    pub at_ms: i64,
    pub tool: String,
}

/// A live subagent / helper spawned by a parent session (Blueprint §4.2, Figure 07).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubagentInfo {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub status: String,
    pub elapsed_s: i64,
}

/// A collision between two sessions (Blueprint §4.1, §4.2, §4.9, Figure 50).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollisionInfo {
    /// "same_file" (red ring, edited within 20 min) or "same_checkout" (quiet warning).
    pub kind: String,
    pub other_id: String,
    pub other_title: String,
    pub file: Option<String>,
}

/// Everything a provider can learn about one session by reading its files or PTY stream.
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

    /// "discovered" or "spawned" (Blueprint §13).
    #[serde(default = "default_origin")]
    pub origin: String,
    /// Host surface: "Terminal", "Claude app", "Codex app", "cmux", "tmux", "iTerm", "AgentWatch PTY".
    #[serde(default = "default_place")]
    pub place: String,

    /// Last `assistant` line ended with stop_reason == "end_turn".
    pub turn_closed: bool,
    /// A `tool_use` block is still open (no matching `tool_result`).
    pub pending_tool: Option<String>,
    /// Epoch millis of the last parsable transcript line.
    pub last_activity_ms: i64,
    /// Epoch millis when the current turn started.
    pub turn_started_ms: Option<i64>,

    /// Last assistant message summary / words (for Finished cards & History).
    #[serde(default)]
    pub last_words: String,
    /// First user prompt of the current or most recent turn.
    #[serde(default)]
    pub first_prompt: String,
    /// Explicit error classification ("auth", "balance", "rate_limit", "trust") — fixes AgentDock Bug #3.
    #[serde(default)]
    pub error_kind: Option<String>,

    /// From the agent's own cost record when it has one; else estimated from model price table.
    pub cost_usd: Option<f64>,
    pub tokens_in: u64,
    pub tokens_out: u64,
    #[serde(default)]
    pub cache_read_tokens: u64,
    #[serde(default)]
    pub cache_write_tokens: u64,

    /// Files edited by this session in the last 20 minutes.
    #[serde(default)]
    pub edited_files: Vec<FileEdit>,
    /// Running or recent subagents attached to this session.
    #[serde(default)]
    pub subagents: Vec<SubagentInfo>,

    /// True when this session is a subagent, not a top-level session.
    pub is_sidechain: bool,
    /// The transcript was parseable at all.
    pub readable: bool,
}

fn default_origin() -> String {
    "discovered".into()
}

fn default_place() -> String {
    "Terminal".into()
}

/// One row of the strip / hover card / sessions grid.
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
    /// Active collisions with other sessions.
    #[serde(default)]
    pub collisions: Vec<CollisionInfo>,
    /// Messages queued for this session when its current turn finishes (`when_done`).
    #[serde(default)]
    pub queued_messages: Vec<String>,
    /// True if the user has already marked this finished turn as seen.
    #[serde(default)]
    pub seen: bool,
}

/// An option inside an Inbox permission or question card (Blueprint §7.1, Figures 53, 57).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardOption {
    pub number: usize,
    pub label: String,
    #[serde(default)]
    pub description: String,
    /// "allow", "deny", or "choose"
    pub action: String,
}

/// One card in the Inbox queue (SPEC-009, Blueprint §4.4, §7.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxCard {
    pub id: String,
    pub session_id: String,
    pub provider: String,
    /// "permission" | "question" | "plan" | "trust" | "auth" | "balance" | "elsewhere" | "finished"
    pub card_type: String,
    pub title: String,
    pub repo: String,
    pub branch: String,
    pub place: String,
    pub age_s: i64,
    /// Uppercase category header, e.g. "PRINT A TEST STRING" or "LANGUAGE" or "BASH COMMAND".
    pub header: String,
    /// Main text or last words.
    pub body: String,
    /// Optional command or diff preview in a monospace box.
    pub command: Option<String>,
    /// Question prompt, e.g. "Do you want to proceed?" or "Which greeting language?".
    pub question: Option<String>,
    pub options: Vec<CardOption>,
    pub answerable: bool,
    pub can_reply: bool,
    /// "open" | "pending_undo"
    pub state: String,
    pub chosen_label: Option<String>,
    pub undo_remaining_ms: Option<i64>,
}

/// One completed (or failed) turn in History (SPEC-009, Blueprint §4.4, Figure 28).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnRecord {
    pub id: String,
    pub session_id: String,
    pub provider: String,
    pub title: String,
    pub folder: String,
    pub repo: String,
    pub branch: String,
    pub place: String,
    pub model: String,
    pub started_ms: i64,
    pub finished_ms: i64,
    pub duration_s: i64,
    pub last_words: String,
    /// "ok" | "error"
    pub outcome: String,
    pub error_kind: Option<String>,
    pub seen: bool,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub cost_usd: f64,
}
