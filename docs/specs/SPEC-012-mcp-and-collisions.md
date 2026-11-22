# SPEC-012 — File-Edit Tracking, Collisions, File Locks & MCP Server

Status: **specified** (Blueprint §4.9, §10.F9–F10, §16.45, §19.2)
Related: D-006

## 1. File-Edit Tracking & Collision Detection (§10.F10, Figure 50)

Every provider extracts file modifications from tool invocations (`Edit`, `MultiEdit`,
`Write`, `NotebookEdit`, `apply_patch`) with `{ path, at_ms, tool }`:

- **File Collision (`same_file`)**: Two distinct sessions modify the same canonical file path
  within a 20-minute window (`1_200_000 ms`).
  - Both sessions receive a red collision ring on their status dot (`#EF4444`).
  - Each session's card displays: `⚠ <filename>: also changed by "<other title>"`.
- **Checkout Collision (`same_checkout`)**: Two active sessions (`working` or `needsYou`)
  share the exact same `cwd` (and are not in separate git worktrees).
  - Each session's card displays a quieter note: `Same checkout as "<other title>"`.

## 2. MCP Server (`agentwatch-mcp`, §4.9, §19.2)

A stdio JSON-RPC 2.0 MCP server binary (`src-tauri/src/bin/agentwatch-mcp.rs`) that forwards
requests to the running AgentWatch app over `~/Library/Application Support/AgentWatch/mcp.sock`
(with direct state fallback if queried standalone).

Supported tools:
1. `list_agents` (`{}`) → `{ count, agents: [{ id, agent, kind, name, status, folder, place, model, turn_started_at, last_activity_at, queued }] }`
2. `get_agent` (`{ id }`) → full agent detail + `branch`, `repository`, `edited_files`, `queued_messages`, `collision`
3. `who_edits` (`{ path }`) → `{ path, agents: [...], collisions: [...], locks: [...] }`
4. `recent_turns` (`{ limit?, folder?, agent? }`) → `{ turns: [{ id, agent_id, kind, title, folder, branch, place, duration_seconds, finished_at, last_words }] }`
5. `send_message` (`{ agent_id, text, when_done? }`) → types into spawned PTY or supported terminal host, or queues when `when_done: true`
6. `start_agent` (`{ kind, folder, prompt, worktree? }`) → spawns a new session
7. `request_lock` (`{ agent_id, path, ttl_seconds? }`) → advisory file lock (§16.45)
8. `release_lock` (`{ agent_id, path }`) → releases advisory file lock
