# SPEC-006 — OpenCode provider

Status: **verified against a live install, 2026-11-09**
Verification badge: `verified`

## Why this one matters

OpenCode records turn state as **data**, not prose. `session_v2` carries `time_busy` and
`time_idle`, so we do not have to infer "is it working" from message text the way we do for
Claude Code. It is the cleanest provider we have.

It is also the best live test we can run: no config is written, no hook is installed — the
provider only reads a SQLite file.

## Schema (verified)

Database: `~/.local/share/opencode/opencode.db` (override `AGENTWATCH_OPENCODE_DB`).
OpenCode uses WAL, so the file is being written while we read it; `sqlite3 -readonly`
handles that.

| Table | Role |
|---|---|
| `session_v2` | one row per session: `id`, `title`, `directory`, `time_updated`, `time_busy`, `time_idle`, `time_archived`, `parent_id` |
| `session_message` | turns: `type` ∈ {assistant, user, idle, system, synthetic, …}, `seq`, `data` JSON |

The assistant `data` blob carries `time.created` / `time.streamed` / `time.completed`.
A message without `time.completed` is still streaming.

There is also a legacy `session` / `message` / `part` schema; it holds old rows and is
**not** what current versions write. Do not read it.

## Status rules

| Condition | Result |
|---|---|
| `time_busy` newer than `time_idle` | **working** |
| last message is `assistant` with no `time.completed` | **working** (still streaming) |
| otherwise | **done** → **idle** after the quiet threshold |

Rows filtered out: `time_archived` set, `parent_id` set (subagents), older than the active
window, and everything past `LIMIT 64`.

## What we cannot do

OpenCode has **no hook system**, so:

- no `needsYou` from a permission prompt — those live in its TUI only
- no answering, ever, until OpenCode exposes an approval channel

Those two dots are watch-only. The UI says so through the provider badge.

## Performance

One `sqlite3` call per tick for **all** sessions (not one per session), with a correlated
subquery per session for the last message and last incomplete assistant message.
`shortcut:` shelling out rather than linking a SQLite driver — see the note in the source.

Measured: 50 sessions parsed in ~0.2 s of query time.
