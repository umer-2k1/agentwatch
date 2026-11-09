//! The event a hook sends us.
//!
//! The helper binary reads Claude Code's hook payload on stdin and forwards a small,
//! normalised JSON line to our socket. Keeping the normalisation here means the rest of the
//! app never depends on Claude Code's payload shape — a vendor change breaks one file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEvent {
    /// Claude Code's `hook_event_name`: SessionStart, UserPromptSubmit, PreToolUse,
    /// Notification, Stop, SubagentStop, SessionEnd.
    pub event: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    /// Human-readable reason, when the payload has one.
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
}

impl HookEvent {
    /// Does this event mean "the agent is blocked on the user"?
    ///
    /// Only `Notification` qualifies. It is the event Claude Code raises when it wants
    /// attention — a permission prompt or a question. `PreToolUse` fires for *every* tool
    /// call and must not be treated as a blocker, or every agent would look stuck.
    pub fn is_waiting(&self) -> bool {
        matches!(self.event.as_str(), "Notification" | "PermissionRequest")
    }

    /// Does this event mean "it is moving again"?
    pub fn clears_waiting(&self) -> bool {
        matches!(
            self.event.as_str(),
            "UserPromptSubmit" | "PostToolUse" | "Stop" | "SubagentStop" | "SessionEnd"
        )
    }

    pub fn detail(&self) -> String {
        let base = self
            .message
            .clone()
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| "waiting for you".to_string());
        match &self.tool_name {
            Some(tool) if !tool.is_empty() => format!("{base} ({tool})"),
            _ => base,
        }
    }

    /// Session key, matching the provider's id format.
    pub fn key(&self) -> Option<String> {
        self.session_id
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("claude-code:{s}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(json: &str) -> HookEvent {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn notification_is_a_blocker() {
        let e = ev(
            r#"{"event":"Notification","session_id":"abc","message":"Claude needs your permission to use Bash"}"#,
        );
        assert!(e.is_waiting());
        assert_eq!(e.key().as_deref(), Some("claude-code:abc"));
        assert!(e.detail().contains("permission"));
    }

    #[test]
    fn pre_tool_use_is_not_a_blocker() {
        // If this were treated as waiting, every agent would look stuck.
        let e = ev(r#"{"event":"PreToolUse","session_id":"abc","tool_name":"Bash"}"#);
        assert!(!e.is_waiting());
    }

    #[test]
    fn stop_and_prompt_clear_the_blocker() {
        for name in ["UserPromptSubmit", "PostToolUse", "Stop", "SessionEnd"] {
            let e = ev(&format!(r#"{{"event":"{name}","session_id":"abc"}}"#));
            assert!(e.clears_waiting(), "{name} should clear");
        }
    }

    #[test]
    fn missing_message_still_has_a_detail() {
        let e = ev(r#"{"event":"Notification","session_id":"abc"}"#);
        assert_eq!(e.detail(), "waiting for you");
    }

    #[test]
    fn unknown_events_are_inert() {
        let e = ev(r#"{"event":"SomeFutureEvent","session_id":"abc"}"#);
        assert!(!e.is_waiting());
        assert!(!e.clears_waiting());
    }
}
