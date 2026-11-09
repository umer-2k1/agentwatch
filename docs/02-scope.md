# Scope

Scope is expressed as **milestones**, each with a hard definition of done.
Anything not listed in a milestone is out of scope by default.

## v0.1 — See  (milestone `M0.1`)

**Done when:** with Claude Code sessions running on this Mac, AgentWatch shows one dot per
session, correct status (working / needs you / done / idle / dead), updates within 2s, and
survives a restart.

In scope:
- Claude Code transcript reading (`.jsonl`)
- Status resolver + confidence + evidence pointer
- **Process liveness check** (the thing competitors skip)
- Dot strip: position, colour, hover card
- Hot reload: new sessions appear, finished ones leave

Out of scope: hooks, inbox, theming, history, other agents.

## v0.2 — Answer (`M0.2`)  — 80% done

- [x] Hook installer for Claude Code (`~/.claude/settings.json`, backed up, idempotent)
- [x] Unix socket event listener + `agentwatch-hook` helper binary
- [x] `needsYou` derived from hooks, outranking file inference, proven end-to-end
- [x] Inbox: blocked rows with `1` allow / `2` deny
- [x] Answering: helper blocks, app returns a decision, helper prints `hookSpecificOutput`
- [ ] 2-second undo
- [ ] Jump-to-agent for supported terminals

## v0.3 — Look (`M0.3`)

- Design tokens → theme system
- Presets, custom accent, glass↔solid, density, dot shape
- Full window (not just dots), settings UI
- Theme export/import as JSON

## v0.4 — Remember (`M0.4`)

- Append-only span log (state transitions)
- History of finished turns
- Weekly counters: interruptions, total blocked time, median response latency
- Activity heatmap

## v1.0 — Everyone (`M1.0`)

- One adapter per agent: Codex, Cursor, Gemini CLI, OpenCode, Copilot, Antigravity, …
- `verified` / `unverified` badge per agent
- Pre-flight collision API

## Explicitly deferred (not in v1)

| Item | Why |
|---|---|
| Windows | needs a separate platform layer; trait is designed for it (D-002) |
| Answering in desktop apps | requires owning the agent (D-001) |
| Auto-approval by default | refused by design (D-005) |
| Team / multi-user / sync | contradicts local-only promise |
| Mobile | out of scope until desktop is done |
