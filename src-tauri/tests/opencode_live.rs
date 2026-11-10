//! Live test against this machine's real OpenCode database.
//!
//! Ignored by default (it needs a real install). This is the "live test with OpenCode"
//! the project was asked to prove: no hooks, no config writes — pure read.
//!
//!   cargo test --test opencode_live -- --ignored --nocapture

use agentwatch_lib::model::Status;
use agentwatch_lib::providers::{opencode::OpenCodeProvider, Provider};

#[test]
#[ignore]
fn reads_a_real_opencode_install() {
    let db = std::env::var("HOME")
        .map(|h| format!("{h}/.local/share/opencode/opencode.db"))
        .unwrap_or_default();
    println!("db: {db}");
    assert!(
        std::path::Path::new(&db).exists(),
        "no OpenCode database at {db}"
    );

    let now = chrono::Utc::now().timestamp_millis();
    let mut p = OpenCodeProvider::new(24 * 30); // wide window: prove the parse, not the recency
    let facts = p.poll(now);

    println!("\n{:<14} {:<34} {:<10} {}", "id", "title", "closed", "age");
    for f in facts.iter().take(12) {
        println!(
            "{:<14} {:<34} {:<10} {}m",
            &f.id[..f.id.len().min(14)],
            truncate(&f.title, 34),
            f.turn_closed,
            (now - f.last_activity_ms) / 60_000
        );
    }
    println!("--- {} OpenCode session(s) ---\n", facts.len());

    assert!(
        !facts.is_empty(),
        "no sessions parsed from a populated database"
    );

    // Every row must be internally consistent: a closed turn has no pending tool.
    for f in &facts {
        if f.turn_closed {
            assert!(
                f.pending_tool.is_none(),
                "closed turn with a pending tool: {:?}",
                f.title
            );
        }
        assert!(
            f.last_activity_ms > 0,
            "session without a timestamp: {:?}",
            f.title
        );
        assert_eq!(f.provider, "opencode");
    }

    // The resolver must agree with the provider's own facts.
    let resolved = agentwatch_lib::debug_sessions(24 * 30);
    let oc: Vec<_> = resolved
        .iter()
        .filter(|s| s.fact.provider == "opencode")
        .collect();
    println!("resolved {} OpenCode session(s)", oc.len());
    for s in oc.iter().take(8) {
        println!(
            "  {:<34} {:<10} {:?}",
            truncate(&s.fact.title, 34),
            s.status.as_str(),
            s.confidence
        );
        assert_ne!(
            s.status,
            Status::Dead,
            "we never observed an OpenCode pid, so we must never claim death"
        );
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n - 1).collect::<String>() + "…"
    }
}
