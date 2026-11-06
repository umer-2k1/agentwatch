# Architecture

## Shape

```
   agents write files            hooks call us
          │                          │
          ▼                          ▼
    ┌───────────┐            ┌──────────────┐
    │  Readers  │            │ Event socket │
    │ (1/file)  │            │  (v0.2)      │
    └─────┬─────┘            └──────┬───────┘
          │  SessionFact             │ HookEvent
          └────────────┬─────────────┘
                       ▼
              ┌─────────────────┐
              │    Resolver     │  + Liveness (PID)  + Clock
              └────────┬────────┘
                       ▼
              ┌─────────────────┐
              │   SessionStore  │  in-memory snapshot
              └────┬────────┬───┘
                   │        │
        ┌──────────▼──┐  ┌──▼────────────┐
        │  Span log   │  │  Tauri events │
        │ (append)    │  │  → UI / tray  │
        └─────────────┘  └───────────────┘
```

## Layers

| Layer | Responsibility | Must not |
|---|---|---|
| `providers/` | turn one agent's on-disk format into `SessionFact` | know about UI, stores, or other providers |
| `resolver` | facts → `Status` + confidence + evidence | do I/O |
| `liveness` | is this PID alive | parse transcripts |
| `store` | hold the current snapshot, emit changes | know about Tauri |
| `ui` | render | compute status |

**Dependency rule: arrows only point down.** `providers` never import `store`. `resolver`
takes a plain struct, so it is unit-testable with no filesystem.

## Why providers are one-file-per-agent

AgentDock keeps every vendor's parser inside a single `AgentStore`. Consequence: each new
agent is a release, and untested parsers ship because there is no seam to test them at.
We invert that (D-003): a `Provider` trait, one file each, one fixture each.

```rust
trait Provider {
    fn id(&self) -> &'static str;          // "claude-code"
    fn label(&self) -> &'static str;       // "Claude Code"
    fn verification(&self) -> Verification; // Verified | Unverified
    fn discover(&self) -> Vec<PathBuf>;    // where its sessions live
    fn read(&self, path: &Path, from: u64) -> ReadOutcome;
}
```

## Threading

- Tauri main thread: UI only.
- One async task: poll loop (v0.1) → per-provider discovery + offset reads.
- One async task: hook socket listener (v0.2).
- Liveness checked on a slower cadence (2 s) than reads (500 ms), because `kill(0)` per PID
  every tick is wasteful.

## Rendering the strip

A borderless, always-on-top, non-activating `NSWindow` pinned to a screen edge. Click-through
where there is nothing to click. Must never steal focus (that is what makes these tools
unbearable).

## Data that outlives a restart

| File | Purpose | Volatile? |
|---|---|---|
| `~/Library/Application Support/AgentWatch/spans.jsonl` | every transition, append-only | no |
| `~/Library/Application Support/AgentWatch/config.json` | settings, theme | no |
| `~/Library/Application Support/AgentWatch/offsets.json` | per-file read offsets | yes, safe to delete |
