//! Session store — the single source of truth for both surfaces (D-004).
//!
//! Owns the providers, the liveness checker and the settings. On each tick it polls every
//! provider, resolves each session, sorts by urgency, and hands the whole snapshot to
//! whoever asks. The UI never computes status; it only renders.

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use std::collections::HashMap;

use crate::hooks::event::{Decision, HookEvent};
use crate::liveness::Liveness;
use crate::model::Session;
use crate::providers::{
    claude::ClaudeProvider, codex::CodexProvider, opencode::OpenCodeProvider, Provider,
};
use crate::resolver::{resolve, Settings};

pub struct Store {
    providers: Vec<Box<dyn Provider>>,
    liveness: Liveness,
    settings: Settings,
    last: Vec<Session>,
    /// session id -> why the hook says it is blocked. Authoritative over file inference.
    waiting: HashMap<String, String>,
    /// session id -> the open channel a `PreToolUse` hook is waiting on.
    ///
    /// Dropping the sender closes the helper's connection, which makes it exit with no
    /// decision — the agent then prompts in its own terminal. That is the correct fallback,
    /// so "forget it" and "never answer" are the same code path.
    pending: HashMap<String, std::sync::mpsc::Sender<Decision>>,
}

/// A provider row for the UI's "what we support" list (D-003 badge).
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub label: String,
    pub verification: String,
    pub sessions: usize,
}

impl Store {
    pub fn new() -> Self {
        Self::with_window(12)
    }

    /// `hours` of transcript history that counts as live.
    pub fn with_window(hours: i64) -> Self {
        Self {
            // v0.1: Claude Code only. Each future agent is one more line here and one more
            // file in providers/ (D-003).
            providers: vec![
                Box::new(ClaudeProvider::with_window(hours)),
                Box::new(CodexProvider::new(hours)),
                Box::new(OpenCodeProvider::new(hours)),
            ],
            liveness: Liveness::new(),
            settings: Settings::default(),
            last: Vec::new(),
            waiting: HashMap::new(),
            pending: HashMap::new(),
        }
    }

    #[allow(dead_code)] // read by the settings UI in M0.3
    pub fn settings(&self) -> Settings {
        self.settings
    }

    pub fn set_idle_after_s(&mut self, secs: i64) {
        self.settings.idle_after_s = secs.max(60);
    }

    /// Fold a hook event into the waiting state.
    ///
    /// This is the only code path that can produce `needsYou`, which is why it is small and
    /// boring: events either set a reason or clear it.
    pub fn apply_hook(&mut self, ev: HookEvent, reply: Option<std::sync::mpsc::Sender<Decision>>) {
        let Some(key) = ev.key() else { return };

        // A decision-capable event both marks the session blocked and holds the channel
        // the answer must travel back on.
        if ev.wants_decision() {
            self.waiting.insert(key.clone(), ev.detail());
            if let Some(tx) = reply {
                self.pending.insert(key, tx);
            }
            return;
        }

        if ev.is_waiting() {
            self.waiting.insert(key, ev.detail());
        } else if ev.clears_waiting() {
            self.waiting.remove(&key);
            self.pending.remove(&key);
        }
    }

    /// Answer a blocked turn. Returns false when there was nothing to answer (the helper
    /// already gave up), so the UI can say so instead of pretending.
    pub fn answer(&mut self, session_id: &str, decision: Decision) -> bool {
        let Some(tx) = self.pending.remove(session_id) else {
            return false;
        };
        let sent = tx.send(decision).is_ok();
        if sent {
            self.waiting.remove(session_id);
        }
        sent
    }

    /// Sessions currently holding an open decision channel.
    pub fn awaiting_decision(&self) -> Vec<String> {
        self.pending.keys().cloned().collect()
    }

    pub fn is_waiting(&self, session_id: &str) -> Option<&str> {
        self.waiting.get(session_id).map(|s| s.as_str())
    }

    /// Poll every provider, resolve, sort. Returns the new snapshot.
    pub fn tick(&mut self) -> &[Session] {
        let now = now_ms();
        let mut out: Vec<Session> = Vec::new();

        // Snapshot of the waiting map, so providers can be polled while borrowing self.
        let waiting = self.waiting.clone();
        let mut liveness = std::mem::take(&mut self.liveness);
        for p in self.providers.iter_mut() {
            for fact in p.poll(now) {
                let live = liveness.check_cwd(&fact.cwd);
                let hook = waiting.get(&fact.id).map(|s| s.as_str());
                out.push(resolve(&fact, &live, hook, now, self.settings));
            }
        }
        self.liveness = liveness;

        // Most urgent first: needs-you, then dead, then working.
        out.sort_by_key(|s| (s.status.urgency(), s.status.as_str(), s.fact.title.clone()));
        self.last = out;
        self.prune_waiting();
        &self.last
    }

    /// Forget blockers for sessions that are no longer on the board. Without this a stale
    /// amber dot could outlive its agent forever.
    fn prune_waiting(&mut self) {
        let live: std::collections::HashSet<String> =
            self.last.iter().map(|s| s.fact.id.clone()).collect();
        if live.is_empty() {
            return; // nothing seen yet this tick; do not wipe real state
        }
        self.waiting.retain(|k, _| live.contains(k));
        // Dropping the sender closes the hook connection, so a vanished session cannot
        // leave an agent waiting on a socket nobody will answer.
        self.pending.retain(|k, _| live.contains(k));
    }

    pub fn snapshot(&self) -> Vec<Session> {
        self.last.clone()
    }

    pub fn providers_info(&self) -> Vec<ProviderInfo> {
        self.providers
            .iter()
            .map(|p| ProviderInfo {
                id: p.id().to_string(),
                label: p.label().to_string(),
                verification: p.verification().as_str().to_string(),
                sessions: self
                    .last
                    .iter()
                    .filter(|s| s.fact.provider == p.id())
                    .count(),
            })
            .collect()
    }
}

pub type SharedStore = Arc<Mutex<Store>>;

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
