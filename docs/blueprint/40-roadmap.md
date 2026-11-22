# Roadmap — our state against the doc's phases

| Phase | Scope | Our status |
|---|---|---|
| **P0 Core** | daemon, store, Claude adapter, PTY spawn, grid, status machine | 🟡 discovery + status done; no PTY, no grid |
| **P1 Inbox** | permission/question cards, 2 s undo, keyboard, notifications, badge | 🟡 backend done, panel missing |
| **P2 Ambient** | edge strip, hover card, ⌥⌥, click-to-focus | ❌ **current work** |
| **P3 Coordination** | edit tracking, collisions, MCP | ❌ |
| **P4 Insight** | History + search, Activity, share | ❌ |
| **P5 Reach** | external hosts, mobile, error states, diagnostics | ❌ |
| **P6 Polish** | onboarding, widgets, policies, licensing | ❌ |

## Next three things, in order

1. **Fix the hook contract** — `PermissionRequest` + `decision.behavior` + `updatedInput.answers`
   (the current `PreToolUse` handler is wrong and would not answer a real Claude prompt).
2. **Edge strip + hover card** — the product's identity; nothing else reads as "ambient".
3. **Inbox panel** — cards, digits, 2 s undo, key legend.
