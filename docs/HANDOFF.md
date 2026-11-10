# Handoff — current state

**Last updated:** 2026-11-10 (session 6)
**Milestone:** M0.1 complete except the edge overlay · M0.2 (hooks) started
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
| Liveness proof | ✅ **verified end-to-end** — `tests/liveness_live.rs` |
| Hook installer | ✅ idempotent, backed up, 7 tests |
| Hook event socket | ✅ listener + event model, 6 tests |
| Hook helper binary | ✅ `agentwatch-hook`, proven through the real binary |
| Hook answering | ✅ `permissionDecision` proven both ways |
| Appearance drawer | ✅ 5 controls, live preview, persisted |
| OpenCode provider | ✅ **verified live** — 50 real sessions, SPEC-006 |
| Codex provider | ✅ 7 tests, 170 real sessions parsed |
| UI: repo-first labels, row detail, thin strip | ✅ (screenshots stale — see trap) |
| Theming | ❌ M0.3 |

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

---

# Session 2 (2026-11-09)

## Shipped
- **`dead` is proven.** `tests/liveness_live.rs` spawns a real process, kills it, and asserts
  the session flips to `dead` with `certain` confidence and evidence `pid NNNN is gone`.
- **Sticky pid memory (D-015).** The reason `dead` works at all: we remember the last pid we
  saw for a directory, because a dead process simply vanishes from the table.
- **Process matching hardened.** `ps` reports full paths, so `comm` now matches on exact
  basename and skips `.app/` bundles — otherwise `Claude Helper` and the desktop app would
  have been treated as agents.
- **Path canonicalisation.** `lsof` returns `/private/tmp/…` where transcripts say `/tmp/…`.
- **Hook installer (SPEC-004)** — merge, backup, idempotent, uninstall. 7 tests.
- **Hook event socket** — `events.sock`, event model, 6 tests. A `Notification` now sets
  `needsYou` in the store and outranks file inference.

## Traps found this session (worth remembering)

| Trap | Detail |
|---|---|
| Copied binaries do not run | a copy of `/bin/sleep` is SIGKILLed instantly (bad signature). **Symlink** it (D-016). |
| `ps comm` is a full path | substring matching catches `Claude Helper`. Match the basename exactly. |
| `/tmp` ≠ `/private/tmp` | canonicalise both sides of every path comparison. |
| Env-var fixtures race | a process-wide env var means parallel integration tests clobber each other. **One test, sequential phases.** |
| Store lifetime matters | liveness memory lives in the store; recreating it per call silently disables `dead`. |

## Next
1. `agentwatch-hook` helper binary (reads stdin payload → socket → exit 0).
2. Answering: helper blocks, app returns a decision, helper prints `hookSpecificOutput`.
3. Edge overlay (M0.1b).
4. Theming (M0.3).

---

# Session 3

## Shipped
- **`agentwatch-hook` helper** — reads Claude's payload on stdin, forwards over the socket,
  **always exits 0**, 400 ms timeouts. A missing app is invisible to the agent.
- **Payload normaliser** (`hooks/forward.rs`) — the only file that knows Claude's field
  names. Tolerates missing fields, ignores garbage, never panics.
- **Proven pipeline** — `tests/hook_pipeline.rs` runs the *real binary* with a *real payload*
  over a *real socket* into the *real store*: dot turns amber, `certain`, then clears on
  `PostToolUse`.
- **Appearance drawer (SPEC-005)** — appearance, accent, dot shape, density, glass. Live
  preview, no Apply button. Persisted in `localStorage`.

## Current counts
45 Rust tests · 6 spec files · 4 ADRs · 18 decisions

## Next
1. **Answering** — helper blocks, app returns a decision, helper prints `hookSpecificOutput`.
   This is the last hard piece of M0.2 and the real differentiator.
2. Inbox list with `1–9` (needs the answer path to be useful).
3. Edge overlay (M0.1b).
4. Move `prefs` from localStorage into `config.json` (M0.3b).

## Traps added this session
| Trap | Detail |
|---|---|
| A hook must never fail the agent | every error path returns, `main` exits 0 unconditionally |
| Missing ack must not block | 400 ms read timeout; the write result is ignored |
| `localStorage` is per-webview | the overlay window will need the same prefs — move to config before that |

---

# Session 4 — answering works

## Shipped
- **Two-way hook protocol.** `PreToolUse` is held open; the app answers; the helper prints
  `{"hookSpecificOutput":{"permissionDecision":"allow"}}` and exits 0.
- **Bounded on both ends**: helper 13 s, app 15 s. The helper always gives up first.
- **Safe fallback proven**: no answer ⇒ empty stdout ⇒ Claude Code's own terminal prompt.
- **Inbox answering UI**: blocked rows show `1 Allow` / `2 Deny`; keys work window-wide.

## Evidence
`tests/hook_pipeline.rs` — 5 tests, including the real binary printing a real
`permissionDecision`, and the timeout case printing nothing.

## Next
1. Live test against a real Claude Code install (everything so far uses a generated payload
   of the verified shape).
2. 2-second undo before the decision is sent.
3. Edge overlay (M0.1b) · theming persistence to `config.json` · more providers.

---

# Session 5 — a second provider, verified live

## Shipped
- **OpenCode provider (SPEC-006).** Reads `session_v2` + `session_message`. Because OpenCode
  stores `time_busy` / `time_idle` as data, `working` vs `done` needs no guessing.
- **Live test passed**: 50 real sessions read from the running install, including the very
  session this work was done in, correctly marked `working`.
- Honest limit recorded: no hooks in OpenCode ⇒ no `needsYou`, no answering, ever.

## Why OpenCode was the right first live test
No config is written and no hook is installed — the provider only reads a SQLite file. It
proves the reader/resolver on real data with zero risk to the user's setup.

## Next
1. Codex provider (`~/.codex/session_index.jsonl` + `sessions/**/rollout-*.jsonl`).
2. Live verification of the Claude hooks — needs explicit user consent to touch
   `~/.claude/settings.json`.
3. Edge overlay · 2s undo · theming persistence.

---

# Session 6

## Shipped
- **Codex provider** — `~/.codex/sessions/**/rollout-*.jsonl`. Signals: `task_started`,
  `task_complete`, `token_count`, `custom_tool_call(_output)`, `session_meta`.
  Reading needs **no subscription** — the rollouts are just files.
- **UI corrections** from reviewing the screenshots:
  - labels are **repo-first** (6 sessions on 3 repos: "shop-web" identifies a session,
    "pricing page layout" does not)
  - the strip is now a thin ambient row of bare dots instead of a duplicate of the list
  - rows are clickable and expand into a detail panel (status, why, folder, pid, quiet)
    instead of relying on a tooltip

## Trap found: the screenshot pipeline broke
Mid-session, `chrome --headless --screenshot=…` began exiting 0 while writing **no file** —
both stable Chrome 157 and Chrome for Testing. `--dump-dom` also returns empty. No fix
found yet; the screenshots in `docs/assets/` therefore predate the UI corrections above.

Workaround for whoever picks this up: open the app itself (`npm run tauri dev`) and capture
the real window, or repair headless Chrome (check for a stale singleton, quarantine on a
fresh Chrome install, or switch to Playwright's screenshot API).

## Next
1. Re-shoot `docs/assets/ui-*.png` against the corrected UI.
2. Jump-to-agent for the terminals we can address (iTerm via tty, tmux, WezTerm, kitty).
3. 2-second undo before a decision is sent.
4. Edge overlay (M0.1b) — where the strip finally earns its place.
