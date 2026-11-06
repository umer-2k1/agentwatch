//! Process liveness — the check competitors skip (D-009, ADR-003).
//!
//! A transcript cannot distinguish *finished* from *crashed* from *Ctrl-C*: a dead agent
//! writes no goodbye. Only the process can tell us. This module answers one question:
//! "is the agent behind this session still running?"
//!
//! v0.1 strategy (no hooks yet): find agent processes with `ps`, resolve each one's working
//! directory with `lsof`, and match sessions by `cwd`.
//!
//! Known limitation: two sessions in the same `cwd` are indistinguishable, so both report
//! alive if either is. In v0.2 the hook gives us the exact PID via `getppid()`, and this
//! scan becomes a fallback.

use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

/// How long a process scan is trusted before re-running it.
const CACHE_MS: u128 = 2000;

#[derive(Default)]
pub struct Liveness {
    last_scan: Option<Instant>,
    /// cwd -> pid of a matching agent process.
    by_cwd: HashMap<String, i32>,
}

pub struct LivenessAnswer {
    /// None = we have no process data at all (never guess "alive").
    pub alive: Option<bool>,
    pub pid: Option<i32>,
}

impl Liveness {
    pub fn new() -> Self {
        Self::default()
    }

    /// Is this pid still running? Uses `kill(pid, 0)`, which performs no signalling.
    pub fn pid_is_alive(pid: i32) -> bool {
        if pid <= 0 {
            return false;
        }
        unsafe {
            // 0 == success (alive). EPERM also means alive but not ours.
            let r = libc::kill(pid, 0);
            if r == 0 {
                return true;
            }
            let err = *libc::__error();
            err == libc::EPERM
        }
    }

    /// Ask about a session by its working directory.
    pub fn check_cwd(&mut self, cwd: &str) -> LivenessAnswer {
        if cwd.is_empty() {
            return LivenessAnswer {
                alive: None,
                pid: None,
            };
        }
        self.refresh_if_stale();
        match self.by_cwd.get(cwd) {
            Some(pid) => {
                if Self::pid_is_alive(*pid) {
                    LivenessAnswer {
                        alive: Some(true),
                        pid: Some(*pid),
                    }
                } else {
                    LivenessAnswer {
                        alive: Some(false),
                        pid: Some(*pid),
                    }
                }
            }
            // No agent process claims this cwd. Could be finished, crashed, or simply
            // running somewhere we cannot see (a GUI app). Report unknown, not dead.
            None => LivenessAnswer {
                alive: None,
                pid: None,
            },
        }
    }

    fn refresh_if_stale(&mut self) {
        let stale = match self.last_scan {
            None => true,
            Some(t) => t.elapsed().as_millis() >= CACHE_MS,
        };
        if !stale {
            return;
        }
        self.last_scan = Some(Instant::now());
        self.by_cwd = scan_agent_processes();
    }
}

/// Agent binaries we look for. Matched against the process command name.
const AGENT_COMMS: &[&str] = &["claude", "codex"];

fn scan_agent_processes() -> HashMap<String, i32> {
    let mut out = HashMap::new();

    let Ok(ps) = Command::new("ps").args(["-axo", "pid=,comm="]).output() else {
        return out;
    };
    let text = String::from_utf8_lossy(&ps.stdout);

    let mut candidates: Vec<i32> = Vec::new();
    for line in text.lines() {
        let mut it = line.trim().splitn(2, ' ');
        let (Some(pid_s), Some(comm)) = (it.next(), it.next()) else {
            continue;
        };
        let Ok(pid) = pid_s.trim().parse::<i32>() else {
            continue;
        };
        let comm_l = comm.to_lowercase();
        if AGENT_COMMS.iter().any(|a| comm_l.contains(a)) {
            candidates.push(pid);
        }
    }

    for pid in candidates {
        if let Some(cwd) = cwd_of_pid(pid) {
            out.insert(cwd, pid);
        }
    }
    out
}

/// Resolve a process's working directory with `lsof`.
fn cwd_of_pid(pid: i32) -> Option<String> {
    let out = Command::new("lsof")
        .args(["-a", "-p", &pid.to_string(), "-d", "cwd", "-Fn"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines()
        .find_map(|l| l.strip_prefix('n').map(|s| s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn our_own_pid_is_alive() {
        assert!(Liveness::pid_is_alive(std::process::id() as i32));
    }

    #[test]
    fn absurd_pid_is_not_alive() {
        assert!(!Liveness::pid_is_alive(2_000_000_000));
    }

    #[test]
    fn zero_pid_is_not_alive() {
        assert!(!Liveness::pid_is_alive(0));
    }

    #[test]
    fn empty_cwd_is_unknown_not_dead() {
        let mut l = Liveness::new();
        assert!(l.check_cwd("").alive.is_none());
    }
}
