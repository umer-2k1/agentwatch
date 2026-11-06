//! The status resolver — SPEC-001.
//!
//! Pure: no I/O, no clock access, no globals. Give it facts, liveness and a time, get a
//! status. This is the file that must never lie, so it is the file that gets the unit tests.

use crate::liveness::LivenessAnswer;
use crate::model::{Confidence, Evidence, EvidenceKind, Session, SessionFact, Status};

/// Thresholds. Defaults are what the UI ships with; the user can change `idle_after_s`
/// in Settings (5/15/30/60 min, or never).
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    /// A finished, quiet session becomes `idle` after this many seconds.
    pub idle_after_s: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            idle_after_s: 15 * 60,
        }
    }
}

/// Resolve one session. Order matters — first match wins. See SPEC-001.
pub fn resolve(
    fact: &SessionFact,
    live: &LivenessAnswer,
    now_ms: i64,
    settings: Settings,
) -> Session {
    let quiet_for_s = ((now_ms - fact.last_activity_ms).max(0)) / 1000;

    let (status, confidence, evidence) = if !fact.readable {
        (
            Status::Unknown,
            Confidence::Unknown,
            Evidence {
                kind: EvidenceKind::Transcript,
                detail: "transcript unreadable".into(),
            },
        )
    } else if live.alive == Some(false) {
        // Rule 1: dead beats everything. This is the state nobody else ships.
        (
            Status::Dead,
            Confidence::Certain,
            Evidence {
                kind: EvidenceKind::Process,
                detail: match live.pid {
                    Some(pid) => format!("pid {} is gone", pid),
                    None => "process is gone".into(),
                },
            },
        )
    } else if fact.pending_tool.is_some() {
        // Rule 3.
        (
            Status::Working,
            Confidence::Inferred,
            Evidence {
                kind: EvidenceKind::Transcript,
                detail: format!(
                    "tool `{}` still running",
                    fact.pending_tool.clone().unwrap_or_default()
                ),
            },
        )
    } else if !fact.turn_closed {
        // Rule 4: no pending tool but the turn never closed — model is thinking.
        (
            Status::Working,
            Confidence::Inferred,
            Evidence {
                kind: EvidenceKind::Transcript,
                detail: "turn in progress".into(),
            },
        )
    } else if quiet_for_s < settings.idle_after_s {
        (
            Status::Done,
            Confidence::Inferred,
            Evidence {
                kind: EvidenceKind::Transcript,
                detail: "turn ended (end_turn)".into(),
            },
        )
    } else {
        // Alive (or unknown) + closed + quiet. If we have no liveness data at all we say so
        // rather than pretend certainty.
        let conf = if live.alive == Some(true) {
            Confidence::Inferred
        } else {
            Confidence::Unknown
        };
        (
            Status::Idle,
            conf,
            Evidence {
                kind: EvidenceKind::Transcript,
                detail: format!("quiet for {}m", quiet_for_s / 60),
            },
        )
    };

    Session {
        fact: fact.clone(),
        status,
        confidence,
        evidence,
        pid: live.pid,
        quiet_for_s,
        turn_age_s: fact
            .turn_started_ms
            .map(|t| ((now_ms - t).max(0)) / 1000),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact() -> SessionFact {
        SessionFact {
            id: "claude-code:abc".into(),
            provider: "claude-code".into(),
            cwd: "/x/app".into(),
            repo: "app".into(),
            readable: true,
            last_activity_ms: 1_000_000,
            turn_closed: true,
            ..Default::default()
        }
    }

    fn alive(v: Option<bool>) -> LivenessAnswer {
        LivenessAnswer { alive: v, pid: Some(42) }
    }

    const NOW: i64 = 1_000_000;

    #[test]
    fn dead_beats_everything() {
        let mut f = fact();
        f.pending_tool = Some("t1".into());
        let s = resolve(&f, &alive(Some(false)), NOW, Settings::default());
        assert_eq!(s.status, Status::Dead);
        assert_eq!(s.confidence, Confidence::Certain);
    }

    #[test]
    fn open_tool_is_working() {
        let mut f = fact();
        f.turn_closed = false;
        f.pending_tool = Some("npm test".into());
        let s = resolve(&f, &alive(Some(true)), NOW, Settings::default());
        assert_eq!(s.status, Status::Working);
    }

    #[test]
    fn just_finished_is_done() {
        let s = resolve(&fact(), &alive(Some(true)), NOW + 5_000, Settings::default());
        assert_eq!(s.status, Status::Done);
    }

    #[test]
    fn finished_and_quiet_is_idle() {
        let s = resolve(
            &fact(),
            &alive(Some(true)),
            NOW + 16 * 60 * 1000,
            Settings::default(),
        );
        assert_eq!(s.status, Status::Idle);
    }

    #[test]
    fn unreadable_is_unknown_not_idle() {
        let mut f = fact();
        f.readable = false;
        let s = resolve(
            &f,
            &alive(None),
            NOW + 60 * 60 * 1000,
            Settings::default(),
        );
        assert_eq!(s.status, Status::Unknown);
        assert_eq!(s.confidence, Confidence::Unknown);
    }

    #[test]
    fn no_liveness_data_is_never_certain() {
        let s = resolve(
            &fact(),
            &alive(None),
            NOW + 60 * 60 * 1000,
            Settings::default(),
        );
        assert_ne!(s.confidence, Confidence::Certain);
    }

    #[test]
    fn urgency_puts_needs_you_first() {
        assert!(Status::NeedsYou.urgency() < Status::Working.urgency());
        assert!(Status::Dead.urgency() < Status::Working.urgency());
    }
}
