//! Live verification against this machine's real Claude Code transcripts.
//!
//! Ignored by default: it depends on the developer's own session files.
//!   cargo test --test live_scan -- --ignored --nocapture
//!
//! M0.1 acceptance criteria covered:
//!   1. real transcripts parse without panicking
//!   2. every parsed session resolves to a status
//!   3. the whole scan is fast enough to poll

use agentwatch_lib::{debug_scan, debug_scan_with_window};
use std::time::Instant;

#[test]
#[ignore]
fn scans_real_sessions() {
    // Default window — may legitimately be empty if nothing ran today.
    let narrow = debug_scan();
    println!("\n[12h window] {} live session(s)", narrow.len());

    // Wide window — proves the parser handles the machine's real history.
    let t0 = Instant::now();
    let wide = debug_scan_with_window(24 * 180);
    let elapsed = t0.elapsed();

    println!("\n--- AgentWatch live scan (180d window) ---");
    println!("{:<30} {:<10} {:<9} {}", "repo", "status", "conf", "quiet");
    for r in wide.iter().take(25) {
        println!(
            "{:<30} {:<10} {:<9} {}",
            truncate(&r.repo, 30),
            r.status,
            r.confidence,
            r.age
        );
    }
    if wide.len() > 25 {
        println!("... {} more", wide.len() - 25);
    }
    println!("--- {} sessions in {:?} ---\n", wide.len(), elapsed);

    assert!(!wide.is_empty(), "no transcripts found at all");
    assert!(
        wide.iter().all(|r| !r.status.is_empty()),
        "every session must resolve to a status"
    );
    // A status board must never take seconds to look at the past.
    assert!(elapsed.as_secs() < 20, "scan too slow: {:?}", elapsed);
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n - 1).collect::<String>() + "…"
    }
}
