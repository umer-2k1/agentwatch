//! Session store — the single source of truth for both surfaces (D-004).
//!
//! Owns the providers, the liveness checker and the settings. On each tick it polls every
//! provider, resolves each session, sorts by urgency, and hands the whole snapshot to
//! whoever asks. The UI never computes status; it only renders.

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::liveness::Liveness;
use crate::model::Session;
use crate::providers::{claude::ClaudeProvider, Provider};
use crate::resolver::{resolve, Settings};

pub struct Store {
    providers: Vec<Box<dyn Provider>>,
    liveness: Liveness,
    settings: Settings,
    last: Vec<Session>,
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
            providers: vec![Box::new(ClaudeProvider::with_window(hours))],
            liveness: Liveness::new(),
            settings: Settings::default(),
            last: Vec::new(),
        }
    }

    #[allow(dead_code)] // read by the settings UI in M0.3
    pub fn settings(&self) -> Settings {
        self.settings
    }

    pub fn set_idle_after_s(&mut self, secs: i64) {
        self.settings.idle_after_s = secs.max(60);
    }

    /// Poll every provider, resolve, sort. Returns the new snapshot.
    pub fn tick(&mut self) -> &[Session] {
        let now = now_ms();
        let mut out: Vec<Session> = Vec::new();

        for p in self.providers.iter_mut() {
            for fact in p.poll(now) {
                let live = self.liveness.check_cwd(&fact.cwd);
                out.push(resolve(&fact, &live, now, self.settings));
            }
        }

        // Most urgent first: needs-you, then dead, then working.
        out.sort_by_key(|s| (s.status.urgency(), s.status.as_str(), s.fact.title.clone()));
        self.last = out;
        &self.last
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
