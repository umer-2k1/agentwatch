# Blueprint — AgentWatch vs AgentDock 1.5.5

Source: the user's functional analysis of AgentDock (22 Nov 2026), captured by driving the
real app. Every requirement below is traceable to that document. This folder is the
**product spec**; `docs/specs/` holds the implementation contracts.

## What AgentDock is

A passive menu-bar "air-traffic control" for AI coding agents. It does not run agents — it
watches session files, renders one dot per agent on the screen edge, and routes answers back
through hooks or terminal automation.

## Parity table — what we have today

| Area | AgentDock | AgentWatch | Notes |
|---|---|---|---|
| Claude Code discovery | ✅ | ✅ | we tail `~/.claude/projects/**/*.jsonl` |
| Codex discovery | ✅ | ✅ | rollouts |
| OpenCode discovery | ❌ broken (v1 schema) | ✅ **v2 schema** | we read `session_v2`; they do not |
| Process liveness / `dead` | ❌ | ✅ | nobody else has this |
| Hook install + answer | ✅ | ✅ | but wrong event: we use `PreToolUse`, the real contract is `PermissionRequest` |
| Edge strip | ✅ | ❌ | **P0 gap** |
| Hover card | ✅ | ❌ | **P0 gap** |
| Inbox panel | ✅ | partial | board rows answer, but no dedicated panel / 2 s undo UI |
| History | ✅ | ❌ | |
| Activity analytics | ✅ | ❌ | |
| Settings (7 tabs) | ✅ | ❌ | we have a 4-control drawer |
| Ask ⌥⌥ | ✅ | ❌ | |
| MCP server | ✅ | ❌ | |
| iPhone page | ✅ | ❌ | |
| Widgets | ✅ | ❌ | |
| Tour / licensing | ✅ | ❌ | |

## What we do differently (the doc's §16)

1. **We own the status machine honestly.** `dead` exists; an abandoned turn is `unknown`,
   never `working` (D-013).
2. **We read OpenCode 2.x.** They are stuck on the legacy schema — the app is blind to
   OpenCode today.
3. **Error states are first-class.** A failed turn is never shown as "Finished.".

## The five design principles (doc §14)

1. Ambient first — status must be readable without opening anything.
2. Keyboard always — every surface reachable without a mouse.
3. One queue for decisions.
4. Never lose a prompt.
5. Explain every failure.
