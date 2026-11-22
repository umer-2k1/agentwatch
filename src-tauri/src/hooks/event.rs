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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "lowercase")]
pub enum Decision {
    Allow,
    Deny,
    /// Choose option N of an `AskUserQuestion` (0-based).
    Choose { index: usize },
    /// No opinion — print nothing and let the agent prompt normally.
    Ask,
}

/// Build the JSON a hook must print on stdout.
///
/// Two different contracts exist and mixing them makes Claude ignore the answer:
///
/// * `PermissionRequest` → `decision.behavior` (+ `updatedInput` when answering a question)
/// * `PreToolUse`        → `permissionDecision`
///
/// We only ever install `PermissionRequest`, because it is the event Claude blocks on.
pub fn hook_output(
    event: &str,
    decision: &Decision,
    payload: &serde_json::Value,
) -> Option<serde_json::Value> {
    let inner = match decision {
        Decision::Ask => return None,

        Decision::Allow => serde_json::json!({ "behavior": "allow" }),

        Decision::Deny => serde_json::json!({
            "behavior": "deny",
            "message": "The user declined this in AgentWatch."
        }),

        Decision::Choose { index } => {
            // Questions arrive as PermissionRequest with tool_name AskUserQuestion.
            // Answering = allow + updatedInput.answers (question text → chosen label).
            let mut updated = payload
                .get("tool_input")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));

            if let Some(questions) = updated.get("questions").and_then(|q| q.as_array()).cloned() {
                if let Some(q) = questions.first() {
                    let label = q
                        .get("options")
                        .and_then(|o| o.as_array())
                        .and_then(|opts| opts.get(*index))
                        .and_then(|o| o.get("label"))
                        .and_then(|l| l.as_str())
                        .unwrap_or("");
                    let question = q
                        .get("question")
                        .and_then(|t| t.as_str())
                        .unwrap_or("answer");
                    if !label.is_empty() {
                        updated["answers"] =
                            serde_json::json!({ question: label });
                    }
                }
            }

            serde_json::json!({ "behavior": "allow", "updatedInput": updated })
        }
    };

    Some(serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": event,
            "decision": inner
        }
    }))
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
    /// Only `PermissionRequest`: it is the hook Claude Code **blocks on** (up to 3600 s)
    /// and reads a `decision.behavior` from. `PreToolUse` fires for every tool call and
    /// `Notification` is fire-and-forget — answering either would do nothing.
    pub fn wants_decision(&self) -> bool {
        self.event == "PermissionRequest"
    }

    /// Is this a question rather than a yes/no permission?
    pub fn is_question(&self) -> bool {
        self.tool_name.as_deref() == Some("AskUserQuestion")
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
    fn only_permission_request_carries_a_decision() {
        assert!(ev(r#"{"event":"PermissionRequest","session_id":"a"}"#).wants_decision());
        assert!(!ev(r#"{"event":"PreToolUse","session_id":"a"}"#).wants_decision());
        assert!(!ev(r#"{"event":"Notification","session_id":"a"}"#).wants_decision());
    }

    #[test]
    fn allow_uses_the_permission_request_contract() {
        let p = serde_json::json!({});
        let out = hook_output("PermissionRequest", &Decision::Allow, &p).unwrap();
        assert_eq!(out["hookSpecificOutput"]["hookEventName"], "PermissionRequest");
        assert_eq!(out["hookSpecificOutput"]["decision"]["behavior"], "allow");
    }

    #[test]
    fn deny_carries_the_message_claude_shows() {
        let p = serde_json::json!({});
        let out = hook_output("PermissionRequest", &Decision::Deny, &p).unwrap();
        assert_eq!(out["hookSpecificOutput"]["decision"]["behavior"], "deny");
        assert!(out["hookSpecificOutput"]["decision"]["message"]
            .as_str()
            .unwrap()
            .contains("declined"));
    }

    #[test]
    fn choosing_an_option_answers_the_question() {
        // The exact shape Claude Code expects for AskUserQuestion (doc §7.1).
        let payload = serde_json::json!({
            "tool_name": "AskUserQuestion",
            "tool_input": { "questions": [{
                "question": "Which greeting language?",
                "options": [{ "label": "English" }, { "label": "German" }, { "label": "French" }]
            }]}
        });
        let out = hook_output("PermissionRequest", &Decision::Choose { index: 2 }, &payload).unwrap();
        assert_eq!(out["hookSpecificOutput"]["decision"]["behavior"], "allow");
        assert_eq!(
            out["hookSpecificOutput"]["decision"]["updatedInput"]["answers"]["Which greeting language?"],
            "French"
        );
    }

    #[test]
    fn ask_emits_nothing_so_the_terminal_still_prompts() {
        let p = serde_json::json!({});
        assert!(hook_output("PermissionRequest", &Decision::Ask, &p).is_none());
    }

    #[test]
    fn unknown_events_are_inert() {
        let e = ev(r#"{"event":"SomeFutureEvent","session_id":"abc"}"#);
        assert!(!e.is_waiting());
        assert!(!e.clears_waiting());
    }
}
