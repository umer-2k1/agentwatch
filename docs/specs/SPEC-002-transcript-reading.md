# SPEC-002 — Reading Claude Code transcripts

Status: **verified against live files on 2026-11-06** (macOS 26.4.1, Claude Code current)

## Location

```
~/.claude/projects/<url-encoded-cwd>/<session-id>.jsonl
```

`<url-encoded-cwd>` is the absolute path with `/` replaced by `-`, e.g.
`/Users/me/code/app` → `-Users-me-code-app`.

One file per session. Append-only, one JSON object per line.

## Line types observed (real sample, 2026-11-06)

| `type` | Carries | Used for |
|---|---|---|
| `assistant` | `message.content[]`, `message.stop_reason`, `message.usage`, `message.model`, `gitBranch`, `cwd`, `sessionId`, `slug`, `timestamp`, `isSidechain` | **status, tokens, branch, title** |
| `user` | `message.content[]` (`tool_result`), `permissionDecision`, `toolUseResult` | closing a pending tool |
| `system` | `subtype`, `stopReason`, `hookInfos`, `preventedContinuation` | boundary signals |
| `cost-state` | `totalCostUSD`, `modelUsage`, `totalLinesAdded/Removed` | **exact cost, no price table needed** |
| `custom-title` | `customTitle` | user-renamed session title |
| `agent-name` | `agentName` | display name |
| `last-prompt` | `lastPrompt` | last thing the user asked |
| `mode`, `attachment`, `api-request`, `api-request-blob`, `atis-latch` | — | ignore |

**Important:** the format has line types beyond `user`/`assistant`. The reader must ignore
unknown types silently rather than error (`SPEC-001`, confidence `unknown` only when the
*whole* line is unparseable).

## Deriving status inputs

| Field | Rule |
|---|---|
| `pendingTool` | last `assistant` line's `content` contains a block `type: "tool_use"`, and no later `user` line contains `tool_result` with the same `tool_use_id` |
| `turnClosed` | last `assistant` line has `message.stop_reason == "end_turn"` |
| `lastActivity` | `timestamp` of the last parsable line |
| `title` | `custom-title` > `agent-name` > `slug` > folder name |
| `model` | last `assistant` line's `message.model` |
| `branch` | `gitBranch` on the last `assistant` line |
| `cost` | last `cost-state` line's `totalCostUSD` |

## Tailing rules (correctness constraints)

1. **Keep a byte offset per file.** Never re-read a whole transcript on every tick.
2. **Buffer the trailing partial line.** JSONL writes are not atomic; a mid-turn read will
   see half a line. Only parse up to the last `\n`.
3. **Detect truncation.** If file size < saved offset, the file was rewritten (rewind/edit
   in Claude Code). Re-parse from zero.
4. **Detect rotation.** Track `(path, inode, size)`. Same path + new inode = new file.
5. **Never block the UI thread.** Reading happens on a background task; the UI receives
   snapshots.

## Performance budget

| Metric | Target |
|---|---|
| Sessions supported | 50 without noticeable cost |
| Transcript re-read per tick | 0 bytes (offset only) |
| Status latency after a line is written | < 500 ms |
| CPU when all agents idle | < 1% |

## Acceptance criteria

- [ ] Reader parses all 7 known line types without error
- [ ] Unknown `type` is ignored, not fatal
- [ ] Partial trailing line is buffered, never parsed
- [ ] Truncated file triggers a full re-parse and a correct status
- [ ] A 20 MB transcript is not re-read on every tick
