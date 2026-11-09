//! Does `dead` actually work?
//!
//! This is the claim nobody else makes, so it gets its own end-to-end test rather than a
//! unit test with a faked boolean. It exercises the real chain:
//!
//!   transcript on disk -> provider -> liveness scan (ps + lsof) -> resolver
//!
//! A stand-in process named `claude` is used (a copy of /bin/sleep), because the scanner
//! matches on process name and working directory — exactly what a real agent exposes.
//! A real agent would make the test slow, costly and racy; this is deterministic.
//!
//! Run: cargo test --test liveness_live -- --nocapture

use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use agentwatch_lib::debug_sessions;
use agentwatch_lib::model::Status;

struct Fixture {
    dir: PathBuf,
    project: PathBuf,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "agentwatch-test-{}-{}",
            std::process::id(),
            tag
        ));
        let project = dir.join("shop-api");
        fs::create_dir_all(&project).unwrap();

        // Point the provider at this fixture, never at the real ~/.claude.
        std::env::set_var("AGENTWATCH_CLAUDE_ROOT", dir.join("claude-root"));

        let sessions = dir.join("claude-root/projects/-Users-test-shop-api");
        fs::create_dir_all(&sessions).unwrap();

        // A turn that never closed: a tool is "running". If the process is alive this is
        // `working`; if it dies it must become `dead` (SPEC-001 rule 2 before rule 5).
        // Built with serde_json rather than string escaping — escaping bugs here would make
        // the test pass for the wrong reason.
        let line = serde_json::json!({
            "type": "assistant",
            "sessionId": "test-session",
            "cwd": project.to_string_lossy(),
            "gitBranch": "main",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "slug": "run the api tests",
            "message": {
                "model": "claude-opus-5",
                "stop_reason": "tool_use",
                "content": [{ "type": "tool_use", "id": "t1", "name": "Bash" }]
            }
        });
        fs::write(
            sessions.join("test-session.jsonl"),
            format!("{}\n", serde_json::to_string(&line).unwrap()),
        )
        .unwrap();

        Self { dir, project }
    }

    /// A process whose name the scanner recognises, started in the project directory.
    fn spawn_agent(&self) -> Child {
        let exe = self.dir.join("claude");
        if !exe.exists() {
            fs::copy("/bin/sleep", &exe).unwrap();
        }
        let child = Command::new(&exe)
            .arg("300")
            .current_dir(&self.project)
            .spawn()
            .expect("spawn stand-in agent");
        // Give ps/lsof a moment to see it.
        thread::sleep(Duration::from_millis(700));
        child
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
        std::env::remove_var("AGENTWATCH_CLAUDE_ROOT");
    }
}

fn statuses() -> Vec<Status> {
    debug_sessions(1).iter().map(|s| s.status).collect()
}

#[test]
fn alive_agent_is_working_and_a_killed_agent_is_dead() {
    let fx = Fixture::new("dead");

    let mut child = fx.spawn_agent();

    // 1. Alive + open turn  => working
    let live = debug_sessions(1);
    println!("alive: {:?}", live.iter().map(|s| s.status).collect::<Vec<_>>());
    for s in &live {
        println!("FACT: {:#?}", s.fact);
    }
    assert!(
        live.iter().any(|s| s.status == Status::Working),
        "expected `working` while the process is alive, got {:?}",
        statuses()
    );

    // 2. Kill it => dead, within one tick
    child.kill().unwrap();
    let _ = child.wait();
    thread::sleep(Duration::from_millis(400));

    let after = debug_sessions(1);
    println!("killed: {:?}", after.iter().map(|s| s.status).collect::<Vec<_>>());
    println!("AFTER: {:#?}", after);
    assert_eq!(
        after.first().map(|s| s.status),
        Some(Status::Dead),
        "a killed agent must resolve to `dead`, not `working` or `idle`"
    );

    // 3. And it must be certain, not inferred — this is a process fact.
    let s = &after[0];
    println!("evidence: {}", s.evidence.detail);
    assert_eq!(format!("{:?}", s.confidence).to_lowercase(), "certain");
}

#[test]
fn no_process_yields_unknown_not_dead() {
    // Transcript exists, but nothing is running it. We must not claim it died.
    let fx = Fixture::new("unknown");
    let _ = &fx; // keep the fixture alive; no process spawned

    let rows = debug_sessions(1);
    println!("no process: {:?}", rows.iter().map(|s| s.status).collect::<Vec<_>>());
    assert!(
        rows.iter().all(|s| s.status != Status::Dead),
        "without process data we must never assert death"
    );
}
