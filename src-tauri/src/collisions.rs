//! File-edit collision detection and advisory file locks — SPEC-012.
//!
//! Covers Blueprint §4.1, §4.2, §4.9, §10.F10, §16.45, §18.56, and §19.2.
//!
//! Two collision classes:
//! * `same_file`     — two sessions modified the same file within 20 minutes (red ring).
//! * `same_checkout` — two active sessions operate in the same working directory (quiet warning).

use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::model::{CollisionInfo, Session, Status};

/// Window within which two edits to the same file count as a collision (20 minutes).
pub const COLLISION_WINDOW_MS: i64 = 20 * 60 * 1000;

/// An advisory file lock requested via MCP `request_lock` (Blueprint §10.F9, §16.45).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileLock {
    pub agent_id: String,
    pub path: String,
    pub acquired_ms: i64,
    pub expires_ms: i64,
}

/// Output of the `who_edits` query / MCP tool (Blueprint §19.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhoEditsReport {
    pub path: String,
    pub agents: Vec<WhoEditsAgent>,
    pub collisions: Vec<WhoEditsCollision>,
    pub locks: Vec<FileLock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhoEditsAgent {
    pub agent_id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub files: Vec<String>,
    pub last_edit_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhoEditsCollision {
    pub kind: String,
    pub agent_ids: Vec<String>,
    pub agents: Vec<String>,
    pub files: Vec<String>,
}

/// Populate `session.collisions` across all active sessions.
pub fn annotate_collisions(sessions: &mut [Session], now_ms: i64) {
    let n = sessions.len();
    for s in sessions.iter_mut() {
        s.collisions.clear();
    }
    if n < 2 {
        return;
    }

    for i in 0..n {
        for j in (i + 1)..n {
            if sessions[i].fact.id == sessions[j].fact.id {
                continue;
            }

            // 1. Check `same_file` within COLLISION_WINDOW_MS.
            let mut shared_file: Option<String> = None;
            for ei in &sessions[i].fact.edited_files {
                if now_ms - ei.at_ms > COLLISION_WINDOW_MS {
                    continue;
                }
                for ej in &sessions[j].fact.edited_files {
                    if now_ms - ej.at_ms > COLLISION_WINDOW_MS {
                        continue;
                    }
                    if ei.path == ej.path && (ei.at_ms - ej.at_ms).abs() <= COLLISION_WINDOW_MS {
                        let short = Path::new(&ei.path)
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or(&ei.path)
                            .to_string();
                        shared_file = Some(short);
                        break;
                    }
                }
                if shared_file.is_some() {
                    break;
                }
            }

            let id_i = sessions[i].fact.id.clone();
            let title_i = sessions[i].fact.title.clone();
            let id_j = sessions[j].fact.id.clone();
            let title_j = sessions[j].fact.title.clone();

            if let Some(file) = shared_file {
                sessions[i].collisions.push(CollisionInfo {
                    kind: "same_file".into(),
                    other_id: id_j,
                    other_title: title_j,
                    file: Some(file.clone()),
                });
                sessions[j].collisions.push(CollisionInfo {
                    kind: "same_file".into(),
                    other_id: id_i,
                    other_title: title_i,
                    file: Some(file),
                });
                continue;
            }

            // 2. Check `same_checkout`: same non-empty cwd and both are active.
            let cwd_i = &sessions[i].fact.cwd;
            let cwd_j = &sessions[j].fact.cwd;
            let active_i = matches!(sessions[i].status, Status::Working | Status::NeedsYou);
            let active_j = matches!(sessions[j].status, Status::Working | Status::NeedsYou);
            if !cwd_i.is_empty() && cwd_i == cwd_j && (active_i && active_j) {
                sessions[i].collisions.push(CollisionInfo {
                    kind: "same_checkout".into(),
                    other_id: id_j,
                    other_title: title_j,
                    file: None,
                });
                sessions[j].collisions.push(CollisionInfo {
                    kind: "same_checkout".into(),
                    other_id: id_i,
                    other_title: title_i,
                    file: None,
                });
            }
        }
    }
}

/// Answer `who_edits(path)` for MCP and the UI pre-flight check (Blueprint §19.2).
pub fn who_edits(
    sessions: &[Session],
    query_path: &str,
    locks: &[FileLock],
    now_ms: i64,
) -> WhoEditsReport {
    let mut agents = Vec::new();
    let mut collisions = Vec::new();

    for s in sessions {
        let matching: Vec<_> = s
            .fact
            .edited_files
            .iter()
            .filter(|e| {
                now_ms - e.at_ms <= COLLISION_WINDOW_MS
                    && (query_path.is_empty()
                        || e.path == query_path
                        || e.path.starts_with(query_path)
                        || query_path.starts_with(&s.fact.cwd))
            })
            .collect();

        if !matching.is_empty() || (!query_path.is_empty() && s.fact.cwd == query_path) {
            let last_edit_at = matching.iter().map(|e| e.at_ms).max().unwrap_or(s.fact.last_activity_ms);
            agents.push(WhoEditsAgent {
                agent_id: s.fact.id.clone(),
                name: s.fact.title.clone(),
                kind: s.fact.provider.clone(),
                status: s.status.as_str().to_string(),
                files: matching.iter().map(|e| e.path.clone()).collect(),
                last_edit_at,
            });
        }

        for c in &s.collisions {
            if s.fact.id < c.other_id {
                collisions.push(WhoEditsCollision {
                    kind: c.kind.clone(),
                    agent_ids: vec![s.fact.id.clone(), c.other_id.clone()],
                    agents: vec![s.fact.title.clone(), c.other_title.clone()],
                    files: c.file.iter().cloned().collect(),
                });
            }
        }
    }

    let active_locks = locks
        .iter()
        .filter(|l| l.expires_ms > now_ms && (query_path.is_empty() || l.path.starts_with(query_path)))
        .cloned()
        .collect();

    WhoEditsReport {
        path: query_path.to_string(),
        agents,
        collisions,
        locks: active_locks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Confidence, Evidence, EvidenceKind, FileEdit, SessionFact};

    fn mk_session(id: &str, title: &str, cwd: &str, status: Status, edits: Vec<FileEdit>) -> Session {
        Session {
            fact: SessionFact {
                id: id.into(),
                provider: "claude-code".into(),
                title: title.into(),
                cwd: cwd.into(),
                edited_files: edits,
                readable: true,
                ..Default::default()
            },
            status,
            confidence: Confidence::Certain,
            evidence: Evidence {
                kind: EvidenceKind::Transcript,
                detail: "test".into(),
            },
            pid: Some(100),
            quiet_for_s: 0,
            turn_age_s: None,
            collisions: Vec::new(),
            queued_messages: Vec::new(),
            seen: false,
        }
    }

    #[test]
    fn detects_same_file_collision_within_20_minutes() {
        let now = 2_000_000;
        let mut list = vec![
            mk_session(
                "s1",
                "Add Stripe checkout",
                "/repo",
                Status::NeedsYou,
                vec![FileEdit {
                    path: "/repo/src/SessionStore.swift".into(),
                    at_ms: now - 60_000,
                    tool: "Edit".into(),
                }],
            ),
            mk_session(
                "s2",
                "Fix the flaky login test",
                "/repo",
                Status::Working,
                vec![FileEdit {
                    path: "/repo/src/SessionStore.swift".into(),
                    at_ms: now - 120_000,
                    tool: "Write".into(),
                }],
            ),
        ];

        annotate_collisions(&mut list, now);
        assert_eq!(list[0].collisions.len(), 1);
        assert_eq!(list[0].collisions[0].kind, "same_file");
        assert_eq!(list[0].collisions[0].file.as_deref(), Some("SessionStore.swift"));
        assert_eq!(list[1].collisions[0].other_title, "Add Stripe checkout");
    }

    #[test]
    fn detects_same_checkout_when_both_active_in_same_cwd() {
        let now = 2_000_000;
        let mut list = vec![
            mk_session("s1", "Agent A", "/repo/same", Status::Working, vec![]),
            mk_session("s2", "Agent B", "/repo/same", Status::NeedsYou, vec![]),
        ];
        annotate_collisions(&mut list, now);
        assert_eq!(list[0].collisions.len(), 1);
        assert_eq!(list[0].collisions[0].kind, "same_checkout");
    }
}
