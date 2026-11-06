# Handoff — current state

**Last updated:** 2026-11-06 (session 1)
**Milestone:** M0.1 "See" — **shipped to the board, edge overlay still pending**
**Build status:** ✅ compiling, 18 unit tests + 1 live test passing, app launches

## What exists right now

| Area | State |
|---|---|
| Docs spine | ✅ vision, scope, 13 decisions, 3 specs, 3 ADRs, milestone, handoff |
| Claude Code reader | ✅ incremental JSONL tailer, 5 tests |
| Liveness | ✅ `kill(0)` + `ps`/`lsof` scan, 4 tests |
| Resolver | ✅ SPEC-001 with rule 3a, 9 tests |
| Store | ✅ one snapshot, urgency sort, provider registry |
| Board UI | ✅ six states, evidence tooltips, provider badge |
| Edge overlay | ❌ not built (M0.1b) |
| Everything else | ❌ v0.2+ |

## Evidence

`docs/assets/m0.1-board.png` — the board rendering all six states against fixture data.
Captured by rendering the real `index.html`/`styles.css`/`main.js` in headless Chrome with
a mocked `window.__TAURI__` (`/tmp/aw-preview`, throwaway).

## Two bugs found by testing against real files (both fixed)

1. **246 sessions shown.** Claude Code keeps every transcript ever written. Fixed with the
   active window (D-012). First tick went from 11 s to ~0 s.
2. **Months-old abandoned transcripts shown as `working`.** Because `turnClosed == false`
   was read as "in progress". Fixed with rule 3a (D-013): open turn + no process + quiet
   ⇒ `unknown`, not `working`.

Both are recorded in `03-decisions.md`. This is exactly why the live test exists.

## Next actions, in order

1. **M0.1b — edge overlay.** Borderless, always-on-top, non-activating window. This is what
   the name promises. Tauri: `decorations: false`, `alwaysOnTop`, `skipTaskbar`, and
   `NSPanel`-style non-activating behaviour (needs `objc2` or a small custom NSWindow).
2. **M0.1c — verify liveness for real.** Start a Claude Code session, kill it, watch the dot
   go `dead` within 2 s. Not yet verified against a *live* session (none were running).
3. **M0.2 — hooks.** The installer + socket. Read the hook payload contract from
   `~/.claude/settings.json` docs before writing code.

## Known gaps (honest)

| Gap | Impact | Fix |
|---|---|---|
| No live Claude session during dev | `dead` proven only by unit test | run one, kill it |
| liveness matches by `cwd`, not pid | two sessions in one folder both look alive | hook gives exact pid in M0.2 |
| `AGENTWATCH_ACTIVE_WINDOW_HOURS` undocumented for users | minor | add to settings in M0.3 |
| No theme system yet | board is one fixed dark theme | M0.3 |

## Traps for the next agent

- The transcript format has **9+ line types**; unknown ones must be ignored, never fatal.
- `isSidechain: true` lines belong to subagents — they must not change parent status.
- Claude Code **rewinds** messages, which truncates the file. Handle `size < offset`.
- JSONL writes are not atomic. Always keep the trailing partial line.
- Rebuild is fast (~6 s) after the first Tauri compile; the first is ~4 min.
