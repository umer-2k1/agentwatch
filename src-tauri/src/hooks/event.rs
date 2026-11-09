//! The event a hook sends us.
//!
//! The helper binary reads Claude Code's hook payload on stdin and forwards a small,
//! normalised JSON line to our socket. Keeping the normalisation here means the rest of the
//! app never depends on Claude Code's payload shape — a vendor change breaks one file.

use serde::{Deserialize, Serialize};

/// What the user decided about a blocked tool call.
///
/// `Ask` is the safe default: it hands control back to Claude Code's own terminal prompt,
/// which is exactly what would have happened if AgentWatch were not running at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Allow,
    Deny,
    Ask,
}

impl Decision {
    /// The wire form Claude Code understands for a `PreToolUse` hook.
    pub fn hook_output(&self) -> Option<serde_json::Value> {
        let decision = match self {
            Decision::Allow => "allow",
            Decision::Deny => "deny",
            // Emitting nothing is how a hook says "I have no opinion".
            Decision::Ask => return None,
        };
        Some(serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": decision,
                "permissionDecisionReason": "Answered in AgentWatch"
            }
        }))
    }
}

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

    /// Can this event carry a decision back to the agent?
    ///
    /// Only `PreToolUse` can: it is the hook Claude Code blocks on and reads a
    /// `permissionDecision` from. `Notification` is fire-and-forget, so answering there
    /// would do nothing.
    pub fn wants_decision(&self) -> bool {
        self.event == "PreToolUse"
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
    fn only_pre_tool_use_carries_a_decision() {
        assert!(ev(r#"{"event":"PreToolUse","session_id":"a"}"#).wants_decision());
        assert!(!ev(r#"{"event":"Notification","session_id":"a"}"#).wants_decision());
        assert!(!ev(r#"{"event":"Stop","session_id":"a"}"#).wants_decision());
    }

    #[test]
    fn allow_and_deny_serialise_for_claude() {
        let allow = Decision::Allow.hook_output().unwrap();
        assert_eq!(allow["hookSpecificOutput"]["permissionDecision"], "allow");
        let deny = Decision::Deny.hook_output().unwrap();
        assert_eq!(deny["hookSpecificOutput"]["permissionDecision"], "deny");
    }

    #[test]
    fn ask_emits_nothing_so_the_terminal_still_prompts() {
        // The safe default must be indistinguishable from AgentWatch not being installed.
        assert!(Decision::Ask.hook_output().is_none());
    }

    #[test]
    fn unknown_events_are_inert() {
        let e = ev(r#"{"event":"SomeFutureEvent","session_id":"abc"}"#);
        assert!(!e.is_waiting());
        assert!(!e.clears_waiting());
    }
}
