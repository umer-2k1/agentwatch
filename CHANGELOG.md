# Changelog

## [0.1.0] — 2026-11-06

First working board. Read-only, Claude Code only.

### Added
- Claude Code transcript reader: incremental tailing with per-file byte offsets, partial-line
  buffering, and truncation recovery (rewind-safe).
- Status resolver implementing `docs/specs/SPEC-001`: `working`, `needsYou`, `done`, `idle`,
  `dead`, `unknown`, each with a confidence and an evidence pointer.
- Process liveness via `kill(pid, 0)` plus a `ps`/`lsof` scan that binds sessions to pids.
- Board UI: dot strip, urgency-sorted list, header counts, evidence tooltips, provider
  verification badge.
- 500 ms poll loop emitting snapshots to the UI over Tauri events.
- 18 unit tests, 1 live scan test against real transcripts.

### Fixed
- Old transcripts no longer flood the board: sessions outside a 12 h active window are
  ignored (D-012). First tick went from 11 s to ~0 s.
- Abandoned turns no longer report `working`: an open turn with no live process and no
  activity for 15 min resolves to `unknown` (D-013).
