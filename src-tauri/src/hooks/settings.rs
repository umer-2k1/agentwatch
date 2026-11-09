//! Installing AgentWatch's hooks into Claude Code's settings.
//!
//! Rules, in order of importance:
//!
//! 1. **Never touch a file we cannot restore.** Back up to `<file>.agentwatch-backup` first.
//! 2. **Never clobber the user's other hooks.** Merge, do not replace.
//! 3. **Be idempotent.** Installing twice must not duplicate entries.
//! 4. **Never write without being asked.** This module does nothing on its own; the caller
//!    must pass an explicit path, and the UI asks first.
//!
//! The hook is a command Claude Code runs at a lifecycle event. It receives a JSON payload
//! on stdin. Our helper backs up, writes the event to a socket, and exits 0.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

/// Events we subscribe to. `Notification` is the one that fires when Claude Code wants
/// permission — the signal a transcript can never give us (SPEC-001).
pub const EVENTS: &[&str] = &[
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "Notification",
    "Stop",
    "SubagentStop",
    "SessionEnd",
];

/// Marker we put on every hook entry we own, so we can find and remove ours without
/// guessing from the command string.
const MARKER: &str = "AgentWatch";

#[derive(Debug, PartialEq)]
pub enum InstallOutcome {
    /// Nothing changed: our hooks were already present.
    AlreadyInstalled,
    /// Written. Contains the backup path so the UI can offer "undo".
    Installed { backup: PathBuf },
}

/// Add AgentWatch hooks to a Claude Code settings file.
///
/// `hook_command` is the absolute path to the helper binary.
pub fn install(settings_path: &Path, hook_command: &str) -> Result<InstallOutcome, String> {
    let existing = if settings_path.exists() {
        fs::read_to_string(settings_path)
            .map_err(|e| format!("read {}: {e}", settings_path.display()))?
    } else {
        String::new()
    };

    let mut root: Value = if existing.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(&existing)
            .map_err(|e| format!("{} is not valid JSON: {e}", settings_path.display()))?
    };

    if root.get("hooks") == Some(&json!({})) || root.get("hooks").is_none() {
        root["hooks"] = json!({});
    }
    let hooks = root
        .get_mut("hooks")
        .and_then(|h| h.as_object_mut())
        .ok_or("`hooks` in settings is not an object; refusing to touch it")?;

    let mut changed = false;
    for event in EVENTS {
        let entry = hooks
            .entry((*event).to_string())
            .or_insert_with(|| json!([]));
        let Some(arr) = entry.as_array_mut() else {
            return Err(format!(
                "`hooks.{event}` is not an array; refusing to touch it"
            ));
        };
        if arr.iter().any(|g| group_is_ours(g)) {
            continue;
        }
        arr.push(our_group(hook_command));
        changed = true;
    }

    if !changed {
        return Ok(InstallOutcome::AlreadyInstalled);
    }

    // Backup before the first write, and never overwrite an older backup with a newer
    // (already-modified) file.
    let backup = backup_path(settings_path);
    if settings_path.exists() && !backup.exists() {
        fs::copy(settings_path, &backup).map_err(|e| format!("backup: {e}"))?;
    }

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    let pretty = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    fs::write(settings_path, pretty + "\n").map_err(|e| format!("write: {e}"))?;

    Ok(InstallOutcome::Installed { backup })
}

/// Remove our entries, leaving everything else exactly as it was.
pub fn uninstall(settings_path: &Path) -> Result<bool, String> {
    if !settings_path.exists() {
        return Ok(false);
    }
    let text = fs::read_to_string(settings_path).map_err(|e| e.to_string())?;
    let mut root: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    let Some(hooks) = root.get_mut("hooks").and_then(|h| h.as_object_mut()) else {
        return Ok(false);
    };

    let mut changed = false;
    for (_event, entry) in hooks.iter_mut() {
        if let Some(arr) = entry.as_array_mut() {
            let before = arr.len();
            arr.retain(|g| !group_is_ours(g));
            if arr.len() != before {
                changed = true;
            }
        }
    }
    // Drop event keys we emptied.
    let empties: Vec<String> = hooks
        .iter()
        .filter(|(_, v)| v.as_array().map(|a| a.is_empty()).unwrap_or(false))
        .map(|(k, _)| k.clone())
        .collect();
    for k in empties {
        hooks.remove(&k);
    }

    if changed {
        let pretty = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
        fs::write(settings_path, pretty + "\n").map_err(|e| e.to_string())?;
    }
    Ok(changed)
}

pub fn backup_path(settings_path: &Path) -> PathBuf {
    let mut s = settings_path.as_os_str().to_os_string();
    s.push(".agentwatch-backup");
    PathBuf::from(s)
}

fn our_group(hook_command: &str) -> Value {
    json!({
        "matcher": "*",
        "_source": MARKER,
        "hooks": [{ "type": "command", "command": hook_command }]
    })
}

fn group_is_ours(group: &Value) -> bool {
    if group.get("_source").and_then(|s| s.as_str()) == Some(MARKER) {
        return true;
    }
    // Also recognise entries written by an older version that had no marker.
    group
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|arr| {
            arr.iter().any(|h| {
                h.get("command")
                    .and_then(|c| c.as_str())
                    .map(|c| c.contains("agentwatch-hook"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Read the current hooks map, for the UI's "what will change" preview.
pub fn describe(settings_path: &Path) -> Result<Map<String, Value>, String> {
    if !settings_path.exists() {
        return Ok(Map::new());
    }
    let text = fs::read_to_string(settings_path).map_err(|e| e.to_string())?;
    let root: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    Ok(root
        .get("hooks")
        .and_then(|h| h.as_object())
        .cloned()
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static N: AtomicU32 = AtomicU32::new(0);

    fn tmp() -> PathBuf {
        let n = N.fetch_add(1, Ordering::SeqCst);
        let d = std::env::temp_dir().join(format!("aw-hooks-{}-{}", std::process::id(), n));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn read(p: &Path) -> Value {
        serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
    }

    #[test]
    fn creates_settings_when_absent() {
        let d = tmp();
        let p = d.join("settings.json");
        let out = install(
            &p,
            "/Applications/AgentWatch.app/Contents/MacOS/agentwatch-hook",
        )
        .unwrap();
        assert!(matches!(out, InstallOutcome::Installed { .. }));

        let v = read(&p);
        for e in EVENTS {
            assert!(v["hooks"][e].is_array(), "missing event {e}");
        }
    }

    #[test]
    fn keeps_user_hooks_intact() {
        let d = tmp();
        let p = d.join("settings.json");
        fs::write(
            &p,
            r#"{
              "model": "opus",
              "hooks": {
                "Stop": [
                  { "matcher": "*", "hooks": [ { "type": "command", "command": "/usr/local/bin/mine" } ] }
                ]
              }
            }"#,
        )
        .unwrap();

        install(&p, "/x/agentwatch-hook").unwrap();
        let v = read(&p);

        assert_eq!(v["model"], "opus", "unrelated settings must survive");
        let stop = v["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 2, "user hook + ours");
        assert!(stop
            .iter()
            .any(|g| g["hooks"][0]["command"] == "/usr/local/bin/mine"));
        assert!(stop.iter().any(|g| g["_source"] == MARKER));
    }

    #[test]
    fn is_idempotent() {
        let d = tmp();
        let p = d.join("settings.json");
        install(&p, "/x/agentwatch-hook").unwrap();
        let second = install(&p, "/x/agentwatch-hook").unwrap();
        assert_eq!(second, InstallOutcome::AlreadyInstalled);

        let v = read(&p);
        assert_eq!(v["hooks"]["Stop"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn backs_up_the_original_once() {
        let d = tmp();
        let p = d.join("settings.json");
        fs::write(&p, r#"{"model":"opus"}"#).unwrap();

        install(&p, "/x/agentwatch-hook").unwrap();
        let b = backup_path(&p);
        assert!(b.exists());
        assert_eq!(read(&b)["model"], "opus", "backup is the pre-change file");

        // A second install must not overwrite the pristine backup.
        fs::remove_file(&p).unwrap();
        install(&p, "/x/agentwatch-hook").unwrap();
        assert_eq!(read(&b)["model"], "opus");
    }

    #[test]
    fn uninstall_removes_only_ours() {
        let d = tmp();
        let p = d.join("settings.json");
        fs::write(
            &p,
            r#"{"hooks":{"Stop":[{"matcher":"*","hooks":[{"type":"command","command":"/usr/local/bin/mine"}]}]}}"#,
        )
        .unwrap();

        install(&p, "/x/agentwatch-hook").unwrap();
        assert!(uninstall(&p).unwrap());

        let v = read(&p);
        let stop = v["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 1);
        assert_eq!(stop[0]["hooks"][0]["command"], "/usr/local/bin/mine");
        // Events only we used are gone entirely.
        assert!(v["hooks"].get("Notification").is_none());
    }

    #[test]
    fn refuses_to_touch_malformed_hooks() {
        let d = tmp();
        let p = d.join("settings.json");
        fs::write(&p, r#"{"hooks":{"Stop":"not-an-array"}}"#).unwrap();
        assert!(install(&p, "/x/agentwatch-hook").is_err());
        // And the file is untouched.
        assert_eq!(read(&p)["hooks"]["Stop"], "not-an-array");
    }

    #[test]
    fn creates_backup_none_when_no_file() {
        let d = tmp();
        let p = d.join("settings.json");
        install(&p, "/x/agentwatch-hook").unwrap();
        assert!(!backup_path(&p).exists(), "nothing to back up");
    }
}
