# SPEC-001 — Status model

Status: **implemented in v0.1**
Related: D-001, D-009, D-011

## Why this spec exists

A status board is only useful if it never lies. Every rule below exists to make the answer
*correct or explicitly unknown* — never a confident guess.

## The five states

| State | Colour | Meaning |
|---|---|---|
| `working` | blue | mid-turn, model or tool is running |
| `needsYou` | amber | blocked on a question or permission prompt |
| `done` | green | turn ended cleanly, not yet seen |
| `idle` | grey | at the prompt, no turn in progress |
| `dead` | red-grey | process is gone |

`dead` is the state AgentDock cannot produce, because it never checks the process (D-009).

## Inputs

| Input | Source | Precedence |
|---|---|---|
| `turnClosed` | transcript: last assistant line has `stop_reason: "end_turn"` | 2 |
| `pendingTool` | transcript: last block is `tool_use` with no matching `tool_result` | 2 |
| `hookWaiting` | hook event: `PermissionRequest` / question, not yet resolved | **1 (authoritative)** |
| `alive` | process check on the recorded PID | **1 (authoritative)** |
| `quietFor` | now − last transcript timestamp | 3 |

**Precedence rule:** liveness and hooks beat file inference. When they disagree, the hook
wins, because a hook is a fact and a file is a trace.

## Resolution algorithm

Evaluated per session, in this order. First match wins.

```
1. if !alive                                    -> dead
2. if hookWaiting                               -> needsYou
3. if pendingTool                               -> working
4. if !turnClosed                               -> working   (model thinking)
5. if quietFor < idleAfter                      -> done
6. else                                         -> idle
```

Notes:
- Rule 1 before everything: a dead agent is dead regardless of what the file says.
- Rule 5 exists because `end_turn` alone cannot distinguish "finished" from "finished and
  you've seen it". `done` becomes `idle` after the quiet threshold (default 15 min),
  configurable 5/15/30/60/never.
- `done` and `needsYou` clear when the user acknowledges them; that is UI state, not status.

## Confidence

Every status carries a confidence value, and the UI must show it.

| Confidence | When |
|---|---|
| `certain` | derived from a hook, or from liveness |
| `inferred` | derived from transcript content only |
| `unknown` | transcript unreadable, or no liveness data |

`unknown` renders as a grey dot with a question mark. It is never silently rendered as
`idle` — that is the failure mode that makes users stop trusting the board.

## Subagent filtering

Transcript lines with `isSidechain: true` belong to a subagent, not the parent session.
They update timestamps but **must not** change the parent's status. (Reason: a parent
waiting on a subagent is `working`, and a subagent's `end_turn` would otherwise mark the
parent `done`.)

## Evidence

Every status carries a pointer to what produced it: transcript line number, hook event id,
or pid. The UI's "why?" popover shows it. This is how a user disputes a wrong dot.

## Acceptance criteria

- [ ] 3 live sessions show 3 dots with distinct, correct statuses
- [ ] Killing an agent flips its dot to `dead` within 2 s
- [ ] Finishing a turn flips to `done`, then `idle` after the threshold
- [ ] A half-written JSON line never crashes the reader and never changes status
- [ ] A subagent finishing does not mark its parent `done`
- [ ] Unreadable transcript renders `unknown`, not `idle`
