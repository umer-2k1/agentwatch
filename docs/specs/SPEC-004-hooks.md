# SPEC-004 — Hooks

Status: **installer + event ingestion shipped; answering not yet**
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

## Not yet built

- The helper binary itself (`agentwatch-hook`) — must be shipped next to the app binary.
- **Answering.** The hard part: our helper blocks, the app returns a decision, the helper
  prints `hookSpecificOutput` and exits 0. Until then the amber dot is informative only.
- `PermissionRequest` is accepted in the event model but Claude Code's exact naming must be
  verified against a live install before we rely on it.
