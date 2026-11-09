//! Does `dead` actually work?
//!
//! This is the claim nobody else makes, so it gets an end-to-end test rather than a unit
//! test with a faked boolean. It exercises the real chain:
//!
//!   transcript on disk -> provider -> liveness scan (ps + lsof) -> resolver
//!
//! A stand-in process named `claude` is used (a **symlink** to /bin/sleep — a copy loses
//! its code signature and macOS kills it instantly). The scanner matches on process name
//! and working directory, which is exactly what a real agent exposes, so every line that
//! matters runs for real.
//!
//! **One test, not two.** The provider root is configured through a process-wide env var,
//! so two parallel tests would clobber each other's fixture. Sequential phases instead.
//!
//! Run: cargo test --test liveness_live -- --nocapture

use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use agentwatch_lib::model::Status;
use agentwatch_lib::store::Store;

struct Fixture {
    dir: PathBuf,
    project: PathBuf,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("agentwatch-test-{}-{}", std::process::id(), tag));
        let project = dir.join("shop-api");
        fs::create_dir_all(&project).unwrap();

        // Point the provider at this fixture, never at the real ~/.claude.
        std::env::set_var("AGENTWATCH_CLAUDE_ROOT", dir.join("claude-root"));

        let sessions = dir.join("claude-root/projects/-Users-test-shop-api");
        fs::create_dir_all(&sessions).unwrap();

        // A turn that never closed: a tool is "running". Alive => working, killed => dead.
        // Built with serde_json so an escaping bug cannot make this pass for the wrong reason.
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

    fn spawn_agent(&self) -> Child {
        let exe = self.dir.join("claude");
        if !exe.exists() {
            std::os::unix::fs::symlink("/bin/sleep", &exe).unwrap();
        }
        let child = Command::new(&exe)
            .arg("300")
            .current_dir(&self.project)
            .spawn()
            .expect("spawn stand-in agent");
        thread::sleep(Duration::from_millis(700)); // let ps/lsof see it
        child
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // The agent may still be running if an assertion failed early.
        let _ = Command::new("pkill")
            .args(["-f", &self.dir.to_string_lossy()])
            .status();
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// Tick a *single* store, the way the app does. Recreating the store between phases would
/// wipe the liveness memory that makes `dead` reachable at all.
fn tick(store: &mut Store) -> Vec<agentwatch_lib::model::Session> {
    store.tick().to_vec()
}

#[test]
fn liveness_end_to_end() {
    // ---- Phase 1: alive + open turn => working -------------------------------
    // Fixture first: the provider reads its root (env) when the store is constructed.
    let fx = Fixture::new("alive");
    let mut store = Store::with_window(1);
    let mut child = fx.spawn_agent();

    let live = tick(&mut store);
    println!(
        "alive  -> {:?} (pid {:?})",
        live.iter().map(|s| s.status).collect::<Vec<_>>(),
        live.iter().map(|s| s.pid).collect::<Vec<_>>()
    );

    assert_eq!(
        live.first().map(|s| s.status),
        Some(Status::Working),
        "a live agent with an open turn must be `working`"
    );
    assert!(
        live.first().and_then(|s| s.pid).is_some(),
        "liveness must bind the session to a pid"
    );

    // ---- Phase 2: killed => dead, within one tick ----------------------------
    child.kill().unwrap();
    let _ = child.wait();
    thread::sleep(Duration::from_millis(300));

    let after = tick(&mut store);
    println!(
        "killed -> {:?} ({} )",
        after.iter().map(|s| s.status).collect::<Vec<_>>(),
        after
            .first()
            .map(|s| s.evidence.detail.clone())
            .unwrap_or_default()
    );

    assert_eq!(
        after.first().map(|s| s.status),
        Some(Status::Dead),
        "a killed agent must resolve to `dead`, not `working` or `idle`"
    );
    // A process fact, not a guess.
    assert_eq!(
        format!("{:?}", after[0].confidence).to_lowercase(),
        "certain",
        "death must be `certain`, never `inferred`"
    );

    drop(fx);

    // ---- Phase 3: never had a process => never `dead` ------------------------
    let fx2 = Fixture::new("noproc");
    let _ = &fx2; // transcript exists, nothing runs it

    let mut store2 = Store::with_window(1);
    let rows = tick(&mut store2);
    println!(
        "noproc -> {:?}",
        rows.iter().map(|s| s.status).collect::<Vec<_>>()
    );

    assert!(
        rows.iter().all(|s| s.status != Status::Dead),
        "without having observed a process we must never assert death"
    );
}
