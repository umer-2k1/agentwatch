# Requirements (condensed from the analysis)

`MUST` = parity with AgentDock. `SHOULD` = our improvement.

## Functional

| # | Requirement | Status |
|---|---|---|
| F1 | Zero-setup discovery of Claude Code + OpenCode (+ pluggable adapters) | ✅ done |
| F2 | Spawn and host sessions in embedded PTYs | ❌ later phase |
| F3 | One status machine: starting → working → needs_input(question\|permission\|plan\|trust\|auth) → done → idle, plus error(kind) and exited(code) | ⚠️ partial — we have working/needsYou/done/idle/dead/unknown |
| F4 | Inbox with keyboard answers, 2 s undo, reply composer, free-text answer | ⚠️ backend done, UI missing |
| F5 | Answer permissions for Claude Code (hook `PermissionRequest` → `hookSpecificOutput`) | ⚠️ wrong event today |
| F6 | Ambient status: menu-bar badge, edge strip, hover card, notifications | ⚠️ tray only |
| F7 | Jump to the session / external host | ❌ |
| F8 | History + search; Activity analytics | ❌ |
| F9 | MCP server: list_agents, get_agent, who_edits, recent_turns, send_message, start_agent | ❌ |
| F10 | File-edit tracking + collision detection | ❌ |
| F11 | Mobile companion | ❌ |
| F12 | Session recovery | ❌ |
| F13 | Error surfacing (auth, balance, trust, rate limit) | ❌ |

## Non-functional

| # | Requirement |
|---|---|
| N1 | status latency ≤ 1 s for hook events, ≤ 3 s for file tail |
| N2 | idle CPU < 1 %, memory < 150 MB with 20 agents |
| N3 | everything local; network only for updates/licensing/relay |
| N4 | hook install idempotent, backed up, byte-identical uninstall |
| N5 | adapters tolerate schema drift — probe and show "unsupported version", never silence |
| N6 | host CLI calls version-probed |
| N7 | every surface keyboard-reachable |

## The hook contract we must match (doc §7.1, §19.1)

Claude Code runs our hook with a JSON payload on stdin. For `PermissionRequest` the hook
**blocks up to 3600 s** and must print:

```json
{"hookSpecificOutput":{"hookEventName":"PermissionRequest",
  "decision":{"behavior":"deny","message":"The user declined in AgentWatch."}}}
```

For a question (`tool_name: "AskUserQuestion"`) the answer is returned as:

```json
{"hookSpecificOutput":{"hookEventName":"PermissionRequest",
  "decision":{"behavior":"allow","updatedInput":{"questions":[…],"answers":{"<question>":"<chosen label>"}}}}}
```

Timeouts per event (doc §19.1): `PermissionRequest` 3600 s; every other event 5 s.

**This is the single most important correctness fix in the codebase today.**
