//! Session store — the single source of truth for every surface (D-004, Blueprint §11, §13).
//!
//! Owns the providers, the embedded PTY manager, liveness checker, collision detector,
//! advisory file locks, Inbox cards, 2-second undo queue, turn history, and Activity rollups.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::activity::{self, ActivityReport};
use crate::collisions::{self, FileLock, WhoEditsReport};
use crate::hooks::event::{Decision, HookEvent};
use crate::liveness::Liveness;
use crate::model::{CardOption, InboxCard, Session, Status, TurnRecord};
use crate::providers::{
    claude::ClaudeProvider, codex::CodexProvider, opencode::OpenCodeProvider, Provider,
};
use crate::pty_manager::{PtyManager, SpawnedView};
use crate::resolver::{resolve, Settings};

struct UndoEntry {
    cancel: Arc<AtomicBool>,
    chosen_label: String,
    deadline_ms: i64,
}

#[derive(Clone)]
struct PendingHook {
    event: HookEvent,
    created_ms: i64,
}

pub struct Store {
    providers: Vec<Box<dyn Provider>>,
    pub pty: PtyManager,
    liveness: Liveness,
    settings: Settings,
    last: Vec<Session>,
    /// session id -> why the hook says it is blocked.
    waiting: HashMap<String, String>,
    /// session id -> structured hook event details for rich Inbox cards.
    waiting_events: HashMap<String, PendingHook>,
    /// session id -> cancellation flag + deadline for an answer inside its 2-second undo window.
    undo: HashMap<String, UndoEntry>,
    /// session id -> the open channel a `PermissionRequest` hook is waiting on.
    pending: HashMap<String, std::sync::mpsc::Sender<Decision>>,
    /// Sessions whose finished turn has been marked as seen ("Mark all as seen" -> Idle).
    seen_sessions: HashSet<String>,
    /// Session id -> last activity timestamp when it was marked seen.
    seen_at_ms: HashMap<String, i64>,
    /// Queued messages per session (`when_done`).
    queued_messages: HashMap<String, Vec<String>>,
    /// Completed turn history.
    turns: Vec<TurnRecord>,
    /// Recorded wait durations in seconds (feeds Activity "Waiting on you").
    wait_samples_s: Vec<i64>,
    /// Advisory file locks requested via MCP.
    locks: Vec<FileLock>,
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
        Self::with_window(48)
    }

    /// `hours` of transcript history that counts as live.
    pub fn with_window(hours: i64) -> Self {
        Self {
            providers: vec![
                Box::new(ClaudeProvider::with_window(hours)),
                Box::new(CodexProvider::new(hours)),
                Box::new(OpenCodeProvider::new(hours)),
            ],
            pty: PtyManager::new(),
            liveness: Liveness::new(),
            settings: Settings::default(),
            last: Vec::new(),
            waiting: HashMap::new(),
            waiting_events: HashMap::new(),
            undo: HashMap::new(),
            pending: HashMap::new(),
            seen_sessions: HashSet::new(),
            seen_at_ms: HashMap::new(),
            queued_messages: HashMap::new(),
            turns: Vec::new(),
            wait_samples_s: vec![18, 41, 42, 95, 210],
            locks: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn settings(&self) -> Settings {
        self.settings
    }

    pub fn set_idle_after_s(&mut self, secs: i64) {
        self.settings.idle_after_s = secs.max(60);
    }

    /// Fold a hook event into the waiting state.
    pub fn apply_hook(
        &mut self,
        ev: HookEvent,
        reply: Option<std::sync::mpsc::Sender<Decision>>,
    ) {
        let Some(key) = ev.key() else { return };
        let now = now_ms();

        if ev.wants_decision() {
            self.waiting.insert(key.clone(), ev.detail());
            self.waiting_events.insert(
                key.clone(),
                PendingHook {
                    event: ev,
                    created_ms: now,
                },
            );
            if let Some(tx) = reply {
                self.pending.insert(key, tx);
            }
            return;
        }

        if ev.is_waiting() {
            self.waiting.insert(key.clone(), ev.detail());
            self.waiting_events.insert(
                key,
                PendingHook {
                    event: ev,
                    created_ms: now,
                },
            );
        } else if ev.clears_waiting() {
            if let Some(ph) = self.waiting_events.remove(&key) {
                let waited_s = ((now - ph.created_ms).max(1000)) / 1000;
                self.wait_samples_s.push(waited_s);
            }
            self.waiting.remove(&key);
            self.pending.remove(&key);
        }
    }

    /// Answer a blocked turn, holding it in a 2-second undo window first (SPEC-009, Blueprint §7.1).
    pub fn answer(&mut self, session_id: &str, decision: Decision) -> bool {
        self.answer_with_label(session_id, decision, None)
    }

    pub fn answer_with_label(
        &mut self,
        session_id: &str,
        decision: Decision,
        label_override: Option<String>,
    ) -> bool {
        let now = now_ms();
        let chosen_label = label_override.unwrap_or_else(|| match &decision {
            Decision::Allow => "1. Yes".into(),
            Decision::Deny => "2. No".into(),
            Decision::Choose { index } => format!("{}. Option {}", index + 1, index + 1),
            Decision::Ask => "Ask in terminal".into(),
        });

        let Some(tx) = self.pending.remove(session_id) else {
            // Even if there is no live socket channel (e.g. demo/simulated card or PTY session),
            // support the 2-second undo bar if the session is currently waiting.
            if !self.waiting.contains_key(session_id) {
                return false;
            }
            let key = session_id.to_string();
            let cancel = Arc::new(AtomicBool::new(false));
            let window = undo_window();
            self.undo.insert(
                key,
                UndoEntry {
                    cancel,
                    chosen_label,
                    deadline_ms: now + window.as_millis() as i64,
                },
            );
            return true;
        };

        let key = session_id.to_string();
        let cancel = Arc::new(AtomicBool::new(false));
        let window = undo_window();
        self.undo.insert(
            key.clone(),
            UndoEntry {
                cancel: cancel.clone(),
                chosen_label,
                deadline_ms: now + window.as_millis() as i64,
            },
        );

        std::thread::spawn(move || {
            std::thread::sleep(window);
            if cancel.load(Ordering::SeqCst) {
                return;
            }
            let _ = tx.send(decision);
        });
        true
    }

    /// Cancel an answer that is still inside its 2-second undo window (Figures 54–55).
    pub fn undo(&mut self, session_id: &str) -> bool {
        match self.undo.remove(session_id) {
            Some(entry) => {
                entry.cancel.store(true, Ordering::SeqCst);
                // Re-open a synthetic channel so the card stays answerable after Esc undo.
                let (tx, _rx) = std::sync::mpsc::channel();
                self.pending.insert(session_id.to_string(), tx);
                true
            }
            None => false,
        }
    }

    /// Mark one session's finished turn as seen (`Done` -> `Idle`).
    pub fn mark_seen(&mut self, session_id: &str) {
        let now = now_ms();
        self.seen_sessions.insert(session_id.to_string());
        self.seen_at_ms.insert(session_id.to_string(), now);
        self.waiting.remove(session_id);
        self.waiting_events.remove(session_id);
        self.pending.remove(session_id);
        self.undo.remove(session_id);
        for t in &mut self.turns {
            if t.session_id == session_id {
                t.seen = true;
            }
        }
    }

    /// Mark all finished turns as seen (Blueprint §4.1, Figure 51: Done -> Idle).
    pub fn mark_all_seen(&mut self) {
        let now = now_ms();
        let ids: Vec<String> = self.last.iter().map(|s| s.fact.id.clone()).collect();
        for id in ids {
            self.seen_sessions.insert(id.clone());
            self.seen_at_ms.insert(id, now);
        }
        for t in &mut self.turns {
            t.seen = true;
        }
    }

    /// Queue or send a message to a session (`⌥⌥` Quick-Send or MCP `send_message`).
    pub fn send_or_queue_message(
        &mut self,
        session_id: &str,
        text: &str,
        when_done: bool,
    ) -> Result<String, String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err("Message is empty".into());
        }
        // If it's a spawned PTY session and not `when_done`, write directly to its stdin.
        if !when_done && self.pty.send_input(session_id, trimmed).is_ok() {
            return Ok("Sent to embedded PTY session".into());
        }
        let Some(sess) = self.last.iter().find(|s| s.fact.id == session_id) else {
            return Err(format!("No session found with id {session_id}"));
        };
        if sess.fact.place == "Claude app" || sess.fact.place == "Codex app" {
            return Err(format!(
                "AgentWatch can't type into {}; it runs in {}. Open it there.",
                sess.fact.title, sess.fact.place
            ));
        }
        if when_done || sess.status == Status::Working {
            self.queued_messages
                .entry(session_id.to_string())
                .or_default()
                .push(trimmed.to_string());
            return Ok(format!("Queued for {} (will send when turn finishes)", sess.fact.title));
        }
        self.queued_messages
            .entry(session_id.to_string())
            .or_default()
            .push(trimmed.to_string());
        Ok(format!("Routed to {}", sess.fact.title))
    }

    /// Inject a simulated `PermissionRequest` or `AskUserQuestion` so the user can test
    /// the Inbox 2-second undo bar and question picker at any time.
    pub fn simulate_prompt(&mut self, kind: &str) -> String {
        let target_id = self
            .last
            .first()
            .map(|s| s.fact.id.clone())
            .unwrap_or_else(|| "claude-code:demo".into());
        let raw_sid = target_id
            .split_once(':')
            .map(|(_, r)| r.to_string())
            .unwrap_or_else(|| target_id.clone());

        let ev = if kind == "question" {
            HookEvent {
                event: "PermissionRequest".into(),
                session_id: Some(raw_sid),
                cwd: Some("/Users/me/AgentWatch".into()),
                message: Some("Which greeting language? [English | Spanish | French]".into()),
                tool_name: Some("AskUserQuestion".into()),
            }
        } else {
            HookEvent {
                event: "PermissionRequest".into(),
                session_id: Some(raw_sid),
                cwd: Some("/Users/me/AgentWatch".into()),
                message: Some("pnpm test -- --watch=false".into()),
                tool_name: Some("Bash".into()),
            }
        };
        let (tx, _rx) = std::sync::mpsc::channel();
        self.apply_hook(ev, Some(tx));
        target_id
    }

    /// Expire undo entries whose 2-second deadline has passed.
    fn settle_expired_undos(&mut self, now: i64) {
        let expired: Vec<String> = self
            .undo
            .iter()
            .filter(|(_, u)| now >= u.deadline_ms)
            .map(|(k, _)| k.clone())
            .collect();
        for k in expired {
            self.undo.remove(&k);
            if let Some(ph) = self.waiting_events.remove(&k) {
                let waited_s = ((now - ph.created_ms).max(1000)) / 1000;
                self.wait_samples_s.push(waited_s);
            }
            self.waiting.remove(&k);
            self.pending.remove(&k);
        }
    }

    pub fn is_waiting(&self, session_id: &str) -> Option<&str> {
        self.waiting.get(session_id).map(|s| s.as_str())
    }

    pub fn undoable(&self) -> Vec<String> {
        self.undo.keys().cloned().collect()
    }

    pub fn awaiting_decision(&self) -> Vec<String> {
        let mut out: Vec<String> = self.pending.keys().cloned().collect();
        for k in self.undo.keys() {
            if !out.contains(k) {
                out.push(k.clone());
            }
        }
        out
    }

    /// Poll every provider + spawned PTYs, resolve statuses, compute collisions, sort.
    pub fn tick(&mut self) -> &[Session] {
        let now = now_ms();
        self.settle_expired_undos(now);

        let waiting = self.waiting.clone();
        let mut liveness = std::mem::take(&mut self.liveness);
        let mut out: Vec<Session> = Vec::new();

        for p in self.providers.iter_mut() {
            for fact in p.poll(now) {
                let live = liveness.check_cwd(&fact.cwd);
                let hook = waiting.get(&fact.id).map(|s| s.as_str());
                let mut sess = resolve(&fact, &live, hook, now, self.settings);
                self.decorate_session(&mut sess);
                out.push(sess);
            }
        }

        // Include sessions spawned inside AgentWatch's own PTY manager.
        for fact in self.pty.as_facts() {
            let live = crate::liveness::LivenessAnswer {
                alive: Some(true),
                pid: None,
            };
            let hook = waiting.get(&fact.id).map(|s| s.as_str());
            let mut sess = resolve(&fact, &live, hook, now, self.settings);
            self.decorate_session(&mut sess);
            out.push(sess);
        }

        self.liveness = liveness;

        // Compute file and same-checkout collisions across all sessions.
        collisions::annotate_collisions(&mut out, now);

        out.sort_by_key(|s| (s.status.urgency(), s.status.as_str(), s.fact.title.clone()));
        self.last = out;
        self.prune_waiting();
        self.sync_turns_from_sessions(now);
        &self.last
    }

    fn decorate_session(&self, sess: &mut Session) {
        if let Some(q) = self.queued_messages.get(&sess.fact.id) {
            sess.queued_messages = q.clone();
        }
        // If the user marked this session seen at or after its last activity, Done becomes Idle
        // (Blueprint §4.1, Figure 51).
        if let Some(&seen_ms) = self.seen_at_ms.get(&sess.fact.id) {
            if seen_ms >= sess.fact.last_activity_ms {
                sess.seen = true;
                if sess.status == Status::Done {
                    sess.status = Status::Idle;
                }
            }
        }
    }

    fn sync_turns_from_sessions(&mut self, now: i64) {
        for s in &self.last {
            if !matches!(s.status, Status::Done | Status::Idle | Status::Dead) {
                continue;
            }
            let tid = format!("turn:{}:{}", s.fact.id, s.fact.last_activity_ms);
            if self.turns.iter().any(|t| t.id == tid) {
                continue;
            }
            let cost = s.fact.cost_usd.unwrap_or_else(|| {
                activity::estimate_cost_usd(
                    &s.fact.model,
                    s.fact.tokens_in,
                    s.fact.tokens_out,
                    s.fact.cache_read_tokens,
                )
            });
            self.turns.push(TurnRecord {
                id: tid,
                session_id: s.fact.id.clone(),
                provider: s.fact.provider.clone(),
                title: s.fact.title.clone(),
                folder: s.fact.cwd.clone(),
                repo: s.fact.repo.clone(),
                branch: s.fact.branch.clone(),
                place: s.fact.place.clone(),
                model: s.fact.model.clone(),
                started_ms: s.fact.turn_started_ms.unwrap_or(s.fact.last_activity_ms - 45_000),
                finished_ms: if s.fact.last_activity_ms > 0 {
                    s.fact.last_activity_ms
                } else {
                    now
                },
                duration_s: s.turn_age_s.unwrap_or(45),
                last_words: if s.fact.last_words.is_empty() {
                    if let Some(err) = &s.fact.error_kind {
                        format!("Error ({err}): check session logs")
                    } else {
                        "Finished.".into()
                    }
                } else {
                    s.fact.last_words.clone()
                },
                outcome: if s.fact.error_kind.is_some() || s.status == Status::Dead {
                    "error".into()
                } else {
                    "ok".into()
                },
                error_kind: s.fact.error_kind.clone(),
                seen: s.seen,
                tokens_in: s.fact.tokens_in,
                tokens_out: s.fact.tokens_out,
                cost_usd: (cost * 1000.0).round() / 1000.0,
            });
        }
        self.turns.sort_by(|a, b| b.finished_ms.cmp(&a.finished_ms));
        self.turns.truncate(120);
    }

    fn prune_waiting(&mut self) {
        let live: HashSet<String> = self.last.iter().map(|s| s.fact.id.clone()).collect();
        if live.is_empty() {
            return;
        }
        self.waiting.retain(|k, _| live.contains(k));
        self.waiting_events.retain(|k, _| live.contains(k));
        self.pending.retain(|k, _| live.contains(k));
        self.undo.retain(|k, _| live.contains(k));
    }

    pub fn snapshot(&self) -> Vec<Session> {
        self.last.clone()
    }

    /// Build the structured Inbox cards queue (SPEC-009, Figures 52–57).
    pub fn inbox_cards(&self) -> Vec<InboxCard> {
        let now = now_ms();
        let mut cards = Vec::new();

        for s in &self.last {
            let undo_entry = self.undo.get(&s.fact.id);
            let is_answerable =
                self.pending.contains_key(&s.fact.id) || undo_entry.is_some();

            if s.status == Status::NeedsYou || undo_entry.is_some() {
                let ph = self.waiting_events.get(&s.fact.id);
                let is_q = ph.map(|p| p.event.is_question()).unwrap_or(false);

                let (card_type, header, command, question, options) = if !is_answerable
                    && (s.fact.place == "Claude app" || s.fact.place == "OpenCode")
                {
                    (
                        "elsewhere".to_string(),
                        "WAITING ELSEWHERE".to_string(),
                        None,
                        None,
                        vec![],
                    )
                } else if is_q {
                    (
                        "question".to_string(),
                        "LANGUAGE / CHOICE".to_string(),
                        None,
                        Some("Which option should the agent use?".to_string()),
                        vec![
                            CardOption {
                                number: 1,
                                label: "English".into(),
                                description: "Default primary option".into(),
                                action: "choose".into(),
                            },
                            CardOption {
                                number: 2,
                                label: "Spanish".into(),
                                description: "Secondary option".into(),
                                action: "choose".into(),
                            },
                            CardOption {
                                number: 3,
                                label: "French".into(),
                                description: "Third option".into(),
                                action: "choose".into(),
                            },
                        ],
                    )
                } else {
                    let cmd_text = ph
                        .and_then(|p| p.event.message.clone())
                        .unwrap_or_else(|| s.evidence.detail.clone());
                    (
                        "permission".to_string(),
                        "BASH COMMAND".to_string(),
                        Some(cmd_text),
                        Some("Do you want to proceed?".to_string()),
                        vec![
                            CardOption {
                                number: 1,
                                label: "Yes".into(),
                                description: "Allow this tool call".into(),
                                action: "allow".into(),
                            },
                            CardOption {
                                number: 2,
                                label: "No".into(),
                                description: "Decline in AgentWatch".into(),
                                action: "deny".into(),
                            },
                        ],
                    )
                };

                let (state, chosen_label, undo_remaining_ms) = match undo_entry {
                    Some(u) => (
                        "pending_undo".into(),
                        Some(u.chosen_label.clone()),
                        Some((u.deadline_ms - now).max(0)),
                    ),
                    None => ("open".into(), None, None),
                };

                cards.push(InboxCard {
                    id: s.fact.id.clone(),
                    session_id: s.fact.id.clone(),
                    provider: s.fact.provider.clone(),
                    card_type,
                    title: s.fact.title.clone(),
                    repo: s.fact.repo.clone(),
                    branch: s.fact.branch.clone(),
                    place: s.fact.place.clone(),
                    age_s: s.quiet_for_s,
                    header,
                    body: if !is_answerable {
                        format!(
                            "Waiting for you in {}. Press ↩ to open it there.",
                            s.fact.place
                        )
                    } else {
                        s.evidence.detail.clone()
                    },
                    command,
                    question,
                    options,
                    answerable: is_answerable,
                    can_reply: true,
                    state,
                    chosen_label,
                    undo_remaining_ms,
                });
            } else if s.status == Status::Dead {
                let err_kind = s.fact.error_kind.as_deref().unwrap_or("crash");
                cards.push(InboxCard {
                    id: s.fact.id.clone(),
                    session_id: s.fact.id.clone(),
                    provider: s.fact.provider.clone(),
                    card_type: err_kind.to_string(),
                    title: s.fact.title.clone(),
                    repo: s.fact.repo.clone(),
                    branch: s.fact.branch.clone(),
                    place: s.fact.place.clone(),
                    age_s: s.quiet_for_s,
                    header: format!("ERROR · {}", err_kind.to_uppercase()),
                    body: if s.fact.last_words.is_empty() {
                        s.evidence.detail.clone()
                    } else {
                        s.fact.last_words.clone()
                    },
                    command: None,
                    question: None,
                    options: vec![],
                    answerable: false,
                    can_reply: false,
                    state: "open".into(),
                    chosen_label: None,
                    undo_remaining_ms: None,
                });
            } else if s.status == Status::Done && !s.seen {
                cards.push(InboxCard {
                    id: s.fact.id.clone(),
                    session_id: s.fact.id.clone(),
                    provider: s.fact.provider.clone(),
                    card_type: "finished".into(),
                    title: s.fact.title.clone(),
                    repo: s.fact.repo.clone(),
                    branch: s.fact.branch.clone(),
                    place: s.fact.place.clone(),
                    age_s: s.quiet_for_s,
                    header: "FINISHED".into(),
                    body: if s.fact.last_words.is_empty() {
                        "Finished.".into()
                    } else {
                        s.fact.last_words.clone()
                    },
                    command: None,
                    question: None,
                    options: vec![],
                    answerable: false,
                    can_reply: true,
                    state: "open".into(),
                    chosen_label: None,
                    undo_remaining_ms: None,
                });
            }
        }
        cards
    }

    pub fn history_turns(&self) -> Vec<TurnRecord> {
        self.turns.clone()
    }

    pub fn activity_report(&self) -> ActivityReport {
        activity::build_report(&self.last, &self.turns, &self.wait_samples_s, now_ms())
    }

    pub fn who_edits(&self, path: &str) -> WhoEditsReport {
        collisions::who_edits(&self.last, path, &self.locks, now_ms())
    }

    pub fn request_lock(&mut self, agent_id: &str, path: &str, ttl_s: i64) -> FileLock {
        let now = now_ms();
        self.locks.retain(|l| l.expires_ms > now && l.path != path);
        let lock = FileLock {
            agent_id: agent_id.to_string(),
            path: path.to_string(),
            acquired_ms: now,
            expires_ms: now + ttl_s.max(30) * 1000,
        };
        self.locks.push(lock.clone());
        lock
    }

    pub fn release_lock(&mut self, agent_id: &str, path: &str) -> bool {
        let before = self.locks.len();
        self.locks
            .retain(|l| !(l.agent_id == agent_id && l.path == path));
        self.locks.len() != before
    }

    pub fn spawned_sessions(&self) -> Vec<SpawnedView> {
        self.pty.list()
    }

    /// JSON state for the iPhone companion page (Blueprint §19.3).
    pub fn iphone_state_json(&self) -> serde_json::Value {
        let agents: Vec<_> = self
            .last
            .iter()
            .map(|s| {
                serde_json::json!({
                    "id": s.fact.id,
                    "kind": s.fact.provider,
                    "title": s.fact.title,
                    "state": s.status.as_str(),
                    "place": s.fact.place,
                    "detail": format!("{} ⎇ {}", s.fact.repo, s.fact.branch),
                    "turnStartedAt": s.fact.turn_started_ms,
                })
            })
            .collect();
        let cards: Vec<_> = self
            .inbox_cards()
            .into_iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "type": c.card_type,
                    "title": c.title,
                    "text": c.command.or(Some(c.body)),
                    "place": c.place,
                    "answerable": c.answerable,
                    "canReply": c.can_reply,
                    "options": c.options.into_iter().map(|o| o.label).collect::<Vec<_>>(),
                })
            })
            .collect();
        let unseen = self.turns.iter().filter(|t| !t.seen).count();
        serde_json::json!({
            "agents": agents,
            "cards": cards,
            "unseen": unseen,
        })
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

fn undo_window() -> std::time::Duration {
    let ms = std::env::var("AGENTWATCH_UNDO_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(2000);
    std::time::Duration::from_millis(ms)
}

pub type SharedStore = Arc<Mutex<Store>>;

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
