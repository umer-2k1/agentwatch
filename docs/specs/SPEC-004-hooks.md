# SPEC-004 — Hooks

Status: **complete for Claude Code — install, ingest, and answer, all proven**
Related: D-005, SPEC-001

## Why hooks exist at all

A permission prompt exists only in the terminal's TUI. It is never written to the
transcript. No amount of file reading can find it — so the amber dot is impossible without
a hook. This is the single reason the app installs anything into your agent's config.

## What we install

Into `~/.claude/settings.json`, for these events:

`SessionStart` · `UserPromptSubmit` · `PreToolUse` · `Notification` · `Stop` ·
`SubagentStop` · `SessionEnd`

Every entry carries `"_source": "AgentWatch"` so we can find and remove ours without
guessing from the command string.

**Safety rules (each has a test):**

| Rule | Why |
|---|---|
| Back up to `<file>.agentwatch-backup` before the first write | the user must always be able to get back |
| Never overwrite an existing backup | a second install must not destroy the pristine copy |
| Merge, never replace | other tools' hooks must survive untouched |
| Idempotent | installing twice must not duplicate entries |
| Refuse on malformed `hooks` | better to do nothing than to corrupt a config |
| Never write unless the user asks | the UI asks; the code does not act on its own |

`hooks_uninstall` removes only our entries, leaves untouched every key it did not create,
and deletes the backup-less; the backup file is left on disk deliberately.

## Event contract

The helper forwards one normalised JSON line to `~/Library/Application Support/AgentWatch/events.sock`
(mode `0600`, owned by the user):

```json
{ "event": "Notification", "session_id": "…", "cwd": "…", "message": "…", "tool_name": "Bash" }
```

| Event | Effect |
|---|---|
| `Notification` | **set** waiting (this is the permission prompt / question) |
| `UserPromptSubmit`, `PostToolUse`, `Stop`, `SubagentStop`, `SessionEnd` | **clear** waiting |
| `PreToolUse` | inert — it fires for *every* tool, so treating it as a blocker would make every agent look stuck |

Waiting state outranks every file signal (SPEC-001 rule 3) **except death** — a dead agent
is not waiting on you.

## Staleness

A waiting entry is dropped when its session leaves the board, so an amber dot can never
outlive its agent. Implemented in `Store::prune_waiting`.

## Shipped and proven

`tests/hook_pipeline.rs` runs the **real helper binary** with a **real Claude-shaped payload**,
over a **real socket**, into the **real store**, and asserts the dot turns amber with
`certain` confidence — then clears when a `PostToolUse` arrives.

## Helper contract (`agentwatch-hook`)

| Property | Rule |
|---|---|
| Exit code | **always 0** — a missing app must be invisible to the agent |
| Timeout | 400 ms on both write and ack read |
| stdout | nothing, for now (v0.2 prints a decision here) |
| stdin | the raw Claude Code payload, read to EOF |
| socket | `~/Library/Application Support/AgentWatch/events.sock`, overridable with `AGENTWATCH_SOCKET` |

## Answering (shipped)

`PreToolUse` is the only event that can carry a decision, because it is the only one Claude
Code blocks on. The trip:

```
Claude Code ──stdin payload──▶ agentwatch-hook
                                   │  one line over events.sock
                                   ▼
                              AgentWatch app ──user presses 1 / 2──▶
                                   │  {"decision":"allow"}
                                   ▼
Claude Code ◀──stdout permissionDecision── agentwatch-hook exits 0
```

| Rule | Why |
|---|---|
| Only `PreToolUse` is held open | `Notification` is fire-and-forget; answering there does nothing |
| Helper waits 13 s, app waits 15 s | the **helper must always give up first**, so no agent is left waiting on a socket that will not speak |
| No answer ⇒ print **nothing**, exit 0 | identical to AgentWatch not being installed. The terminal prompt runs. That is the safe default |
| `Decision::Ask` emits no JSON | same guarantee, at the type level |
| Dropping the sender closes the connection | "forget this" and "never answer" are one code path |

`tests/hook_pipeline.rs` proves both directions: an answer produces
`{"hookSpecificOutput":{"permissionDecision":"allow"}}`, and no answer produces empty stdout
with exit 0.

Keys: `1` allow, `2` deny, on the blocked row or from anywhere in the window.

## Not yet built

- Live verification against a real Claude Code install (everything is proven against a
  generated payload of the same shape).
- `PermissionRequest` is accepted in the event model but Claude Code's exact naming must be
  verified before we rely on it.
- Rules / auto-approval (D-005) — the policy engine is deliberately empty.
