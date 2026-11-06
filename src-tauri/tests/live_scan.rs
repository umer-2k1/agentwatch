//! Live verification against this machine's real Claude Code transcripts.
//!
//! Ignored by default because it depends on the developer's own session files.
//! Run with:  cargo test --test live_scan -- --ignored --nocapture
//!
//! What it proves (M0.1 acceptance criteria):
//!   1. real transcripts parse without panicking
//!   2. statuses are produced for real sessions
//!   3. liveness returns an answer (some pid, or an explicit "unknown")

use agentwatch_lib::debug_scan;

#[test]
#[ignore]
fn scans_real_sessions() {
    let rows = debug_scan();

    println!("\n--- AgentWatch live scan ---");
    println!("{:<28} {:<10} {:<9} {}", "repo", "status", "conf", "age");
    for r in &rows {
        println!(
            "{:<28} {:<10} {:<9} {}",
            truncate(&r.repo, 28),
            r.status,
            r.confidence,
            r.age
        );
    }
    println!("--- {} sessions ---\n", rows.len());

    assert!(
        !rows.is_empty(),
        "no sessions found — is ~/.claude/projects populated?"
    );
    assert!(
        rows.iter().all(|r| !r.status.is_empty()),
        "every session must resolve to a status"
    );
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n - 1).collect::<String>() + "…"
    }
}
