# Handoff — current state

**Last updated:** 2026-11-06
**Milestone:** M0.1 "See" — in progress
**Build status:** not yet compiling (docs-first)

## Where we are

Docs spine is complete: vision, scope, decision log (D-001..D-011), two specs, architecture
with three ADRs, and this file. No application code has been written yet beyond the Tauri
scaffold from `create-tauri-app`.

## What is verified, not assumed

- Claude Code transcript location and format — **read from real files on this machine**.
  See `specs/SPEC-002-transcript-reading.md` for the seven line types and which fields we use.
- `cost-state.totalCostUSD` exists ⇒ we do **not** need a hardcoded price table.
- `permissionDecision` appears on `user` lines ⇒ possibly a second signal for approvals;
  needs investigation in M0.2.
- macOS 26.4.1, x86_64, Rust 1.96, Node 22.23.4, CLT installed, no full Xcode.

## Next action

Implement in this order (each is a commit):

1. `src-tauri/Cargo.toml` — rename crate to `agentwatch`, add `notify`, `tokio`, `sysinfo`
2. `src-tauri/src/model.rs`
3. `src-tauri/src/providers/mod.rs`
4. `src-tauri/src/providers/claude.rs`
5. `src-tauri/src/liveness.rs`
6. `src-tauri/src/resolver.rs` (+ unit tests)
7. `src-tauri/src/store.rs`
8. wire into `lib.rs`, emit events
9. frontend strip

## Traps already identified

| Trap | Handling |
|---|---|
| JSONL partial writes | buffer to last `\n` only (SPEC-002) |
| Claude rewinds a message → file shrinks | size < offset ⇒ re-parse from 0 |
| subagent `end_turn` marks parent done | ignore `isSidechain: true` for status |
| PID reuse | store pid + start time |
| Tauri dev server slow first build | expect ~5 min on first `cargo build` |
