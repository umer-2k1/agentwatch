//! Embedded session spawner, ring-buffer output stream, and git worktree isolation — SPEC-010.
//!
//! Covers Blueprint §10.F2, §11.27, §14.30 (Mission Control), and §16.38 (worktree-per-agent).

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use serde::{Deserialize, Serialize};

use crate::model::{SessionFact, Status};

const MAX_LINES: usize = 400;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnedView {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub folder: String,
    pub repo: String,
    pub branch: String,
    pub worktree: Option<String>,
    pub pid: Option<u32>,
    pub running: bool,
    pub exit_code: Option<i32>,
    pub lines: Vec<String>,
    pub detected_state: String,
    pub error_kind: Option<String>,
    pub started_ms: i64,
    pub last_activity_ms: i64,
    pub resume_cmd: String,
}

struct ManagedSession {
    view: SpawnedView,
    stdin: Option<ChildStdin>,
    child: Option<Child>,
}

#[derive(Default, Clone)]
pub struct PtyManager {
    inner: Arc<Mutex<HashMap<String, ManagedSession>>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a new managed agent session (Claude Code, OpenCode, Codex, or Shell), optionally
    /// creating an isolated git worktree first (Blueprint §16.38).
    pub fn spawn(
        &self,
        kind: &str,
        folder: &str,
        prompt: &str,
        create_worktree: bool,
        now_ms: i64,
    ) -> Result<SpawnedView, String> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let resolved_folder = folder.replace('~', &home);
        let base_path = Path::new(&resolved_folder);
        if !base_path.exists() {
            return Err(format!("No folder at {resolved_folder}"));
        }

        let slug = slugify(if prompt.trim().is_empty() { kind } else { prompt });
        let id = format!("{kind}:spawned-{now_ms}");

        // Optional git worktree isolation (§16.38).
        let (work_dir, branch, worktree_path) = if create_worktree {
            create_git_worktree(&resolved_folder, &slug).unwrap_or_else(|_| {
                (resolved_folder.clone(), current_branch(&resolved_folder), None)
            })
        } else {
            (resolved_folder.clone(), current_branch(&resolved_folder), None)
        };

        let repo = Path::new(&work_dir)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("workspace")
            .to_string();

        let (program, args, resume_cmd) = build_command(kind, prompt, &home);

        let mut cmd = Command::new(&program);
        cmd.args(&args)
            .current_dir(&work_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut initial_lines = vec![format!(
            "$ {} {}  (in {})",
            program,
            args.join(" "),
            work_dir
        )];

        let (mut child_opt, stdin_opt, pid_opt, running) = match cmd.spawn() {
            Ok(mut c) => {
                let pid = c.id();
                let stdin = c.stdin.take();
                (Some(c), stdin, Some(pid), true)
            }
            Err(e) => {
                initial_lines.push(format!("[AgentWatch] could not start `{program}`: {e}"));
                (None, None, None, false)
            }
        };

        let title = if prompt.trim().is_empty() {
            format!("{kind} · {repo}")
        } else {
            prompt.trim().chars().take(64).collect()
        };

        let view = SpawnedView {
            id: id.clone(),
            kind: kind.to_string(),
            title,
            folder: work_dir,
            repo,
            branch,
            worktree: worktree_path,
            pid: pid_opt,
            running,
            exit_code: None,
            lines: initial_lines,
            detected_state: if running { "working".into() } else { "idle".into() },
            error_kind: None,
            started_ms: now_ms,
            last_activity_ms: now_ms,
            resume_cmd,
        };

        // Stream stdout & stderr into the ring buffer.
        if let Some(child) = child_opt.as_mut() {
            if let Some(out) = child.stdout.take() {
                let inner_c = self.inner.clone();
                let id_c = id.clone();
                thread::spawn(move || {
                    let reader = BufReader::new(out);
                    for line in reader.lines().flatten() {
                        push_line(&inner_c, &id_c, &line);
                    }
                });
            }
            if let Some(err) = child.stderr.take() {
                let inner_c = self.inner.clone();
                let id_c = id.clone();
                thread::spawn(move || {
                    let reader = BufReader::new(err);
                    for line in reader.lines().flatten() {
                        push_line(&inner_c, &id_c, &line);
                    }
                });
            }
        }

        if let Ok(mut map) = self.inner.lock() {
            map.insert(
                id,
                ManagedSession {
                    view: view.clone(),
                    stdin: stdin_opt,
                    child: child_opt,
                },
            );
        }

        Ok(view)
    }

    /// Write a line to a spawned session's stdin.
    pub fn send_input(&self, id: &str, text: &str) -> Result<(), String> {
        let mut map = self.inner.lock().map_err(|_| "lock poisoned")?;
        let s = map
            .get_mut(id)
            .ok_or_else(|| format!("No spawned session with id {id}"))?;
        s.view.lines.push(format!("> {text}"));
        s.view.last_activity_ms = crate::store::now_ms();
        s.view.detected_state = "working".into();
        if let Some(stdin) = s.stdin.as_mut() {
            let mut line = text.to_string();
            if !line.ends_with('\n') {
                line.push('\n');
            }
            stdin
                .write_all(line.as_bytes())
                .and_then(|_| stdin.flush())
                .map_err(|e| format!("write to stdin failed: {e}"))?;
        }
        Ok(())
    }

    /// Stop a spawned session.
    pub fn stop(&self, id: &str) -> bool {
        let Ok(mut map) = self.inner.lock() else { return false };
        let Some(s) = map.get_mut(id) else { return false };
        if let Some(mut c) = s.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        s.view.running = false;
        s.view.detected_state = "idle".into();
        s.view.lines.push("[AgentWatch] session stopped.".into());
        true
    }

    /// List all spawned sessions for Mission Control.
    pub fn list(&self) -> Vec<SpawnedView> {
        let Ok(mut map) = self.inner.lock() else {
            return Vec::new();
        };
        for s in map.values_mut() {
            if let Some(c) = s.child.as_mut() {
                if let Ok(Some(status)) = c.try_wait() {
                    s.view.running = false;
                    s.view.exit_code = status.code();
                    if s.view.detected_state == "working" {
                        s.view.detected_state = "done".into();
                    }
                }
            }
        }
        let mut out: Vec<SpawnedView> = map.values().map(|s| s.view.clone()).collect();
        out.sort_by(|a, b| b.started_ms.cmp(&a.started_ms));
        out
    }

    /// Export spawned sessions as `SessionFact`s so they appear in the strip, hover card and Inbox.
    pub fn as_facts(&self) -> Vec<SessionFact> {
        self.list()
            .into_iter()
            .map(|v| {
                let last_words = v
                    .lines
                    .iter()
                    .rev()
                    .find(|l| !l.trim().is_empty() && !l.starts_with("$ "))
                    .cloned()
                    .unwrap_or_default();
                let working = v.detected_state == "working" && v.running;
                SessionFact {
                    id: v.id,
                    provider: v.kind,
                    title: v.title,
                    cwd: v.folder,
                    repo: v.repo,
                    branch: v.branch,
                    model: String::new(),
                    origin: "spawned".into(),
                    place: "AgentWatch PTY".into(),
                    turn_closed: !working,
                    pending_tool: if working { Some("pty".into()) } else { None },
                    last_activity_ms: v.last_activity_ms,
                    turn_started_ms: if working { Some(v.started_ms) } else { None },
                    last_words,
                    error_kind: v.error_kind,
                    readable: true,
                    ..Default::default()
                }
            })
            .collect()
    }
}

fn push_line(inner: &Arc<Mutex<HashMap<String, ManagedSession>>>, id: &str, raw: &str) {
    let clean = strip_ansi(raw);
    let Ok(mut map) = inner.lock() else { return };
    let Some(s) = map.get_mut(id) else { return };

    s.view.last_activity_ms = crate::store::now_ms();
    let low = clean.to_lowercase();
    if low.contains("login expired") || low.contains("please run /login") {
        s.view.error_kind = Some("auth".into());
        s.view.detected_state = "dead".into();
    } else if low.contains("insufficient balance") {
        s.view.error_kind = Some("balance".into());
        s.view.detected_state = "dead".into();
    } else if low.contains("do you trust the files") || low.contains("do you want to proceed") {
        s.view.detected_state = "needsYou".into();
    }

    s.view.lines.push(clean);
    if s.view.lines.len() > MAX_LINES {
        let drop_n = s.view.lines.len() - MAX_LINES;
        s.view.lines.drain(0..drop_n);
    }
}

/// Strip basic ANSI escape sequences so terminal output reads cleanly in the Mission Control tile.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if nc.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else if c != '\r' {
            out.push(c);
        }
    }
    out
}

fn slugify(s: &str) -> String {
    let slug: String = s
        .chars()
        .take(24)
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    slug.trim_matches('-').to_string()
}

fn current_branch(dir: &str) -> String {
    Command::new("git")
        .args(["-C", dir, "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|b| !b.is_empty())
        .unwrap_or_else(|| "main".into())
}

fn create_git_worktree(repo_dir: &str, slug: &str) -> Result<(String, String, Option<String>), String> {
    let branch = format!("agentwatch/{slug}");
    let wt_dir = Path::new(repo_dir)
        .join(".worktrees")
        .join(slug)
        .to_string_lossy()
        .to_string();
    let status = Command::new("git")
        .args(["-C", repo_dir, "worktree", "add", "-b", &branch, &wt_dir])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok((wt_dir.clone(), branch, Some(wt_dir)))
    } else {
        Err("git worktree add failed".into())
    }
}

fn build_command(kind: &str, prompt: &str, home: &str) -> (String, Vec<String>, String) {
    match kind {
        "claude-code" | "claude" => {
            let bin = [
                format!("{home}/.local/bin/claude"),
                "/opt/homebrew/bin/claude".into(),
                "claude".into(),
            ]
            .into_iter()
            .find(|p| Path::new(p).exists())
            .unwrap_or_else(|| "claude".into());
            let args = if prompt.trim().is_empty() {
                vec![]
            } else {
                vec!["-p".into(), prompt.to_string()]
            };
            (bin, args, "claude --continue".into())
        }
        "opencode" => {
            let bin = [
                format!("{home}/.opencode/bin/opencode"),
                "/opt/homebrew/bin/opencode".into(),
                "opencode".into(),
            ]
            .into_iter()
            .find(|p| Path::new(p).exists())
            .unwrap_or_else(|| "opencode".into());
            let args = if prompt.trim().is_empty() {
                vec![]
            } else {
                vec!["run".into(), prompt.to_string()]
            };
            (bin, args, "opencode".into())
        }
        "codex" => {
            let args = if prompt.trim().is_empty() {
                vec![]
            } else {
                vec!["exec".into(), prompt.to_string()]
            };
            ("codex".into(), args, "codex".into())
        }
        _ => {
            let args = if prompt.trim().is_empty() {
                vec!["-c".into(), "echo 'AgentWatch shell ready'; pwd".into()]
            } else {
                vec!["-c".into(), prompt.to_string()]
            };
            ("/bin/zsh".into(), args, "/bin/zsh".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_escape_codes_are_stripped() {
        assert_eq!(strip_ansi("\x1b[32mPASS\x1b[0m test"), "PASS test");
    }

    #[test]
    fn spawns_a_shell_command_and_captures_output() {
        let mgr = PtyManager::new();
        let v = mgr
            .spawn("shell", "/tmp", "echo hello-from-pty", false, 1000)
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let list = mgr.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, v.id);
        assert!(list[0].lines.iter().any(|l| l.contains("hello-from-pty")));
    }
}
