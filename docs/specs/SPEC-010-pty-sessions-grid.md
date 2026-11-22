# SPEC-010 — Embedded PTY Sessions & Mission Control Grid

Status: **specified** (Blueprint §10.F2, §11, §14.30, §16.38)
Related: D-001 (extended with optional spawned sessions per Blueprint §11.27)

## 1. Dual-Origin Session Model (Blueprint §11.27)

AgentWatch supports two session origins under one unified `Session` model:

1. `discovered` — zero-setup passive observation of external sessions (`~/.claude/projects`,
   `~/.codex/sessions`, `~/.local/share/opencode/opencode.db`, plus running processes).
2. `spawned` — sessions launched by AgentWatch inside an managed child process / PTY pipe,
   giving AgentWatch direct control over stdin, stdout/stderr ring buffer, prompts,
   worktree isolation, and recovery (`resume_cmd`).

## 2. Spawning & Worktree Isolation

`spawn_session(kind, folder, prompt, create_worktree, model)` supports:

- `kind`: `"claude-code"` | `"opencode"` | `"codex"` | `"shell"`
- `create_worktree`: when `true` and `folder` is inside a git repo, creates a sibling or
  `.worktrees/<slug>` git worktree on a new branch (`agentwatch/<slug>`) before starting the
  process, preventing `same_checkout` collisions by construction (§16.38).
- Keeps a rolling ring buffer (last 400 lines of output) per spawned session, streamed to
  the Mission Control tile.
- Detects terminal-level prompts directly from the output stream:
  - Folder trust (`Do you trust the files in this folder?`) → `needs_input:trust`
  - Auth expired (`Login expired` / `Please run /login`) → `error:auth`
  - Balance error (`Insufficient balance`) → `error:balance`
  - Permission prompt (`Do you want to proceed?`) → `needs_input:permission`

## 3. Mission Control Grid (§14.30)

- Layouts: `1-up`, `2-up`, `4-up` grid of live session tiles, plus filter chips
  (`All`, `Needs you`, `Working`, `Done`, `Errors`, and per-repo chips).
- Each tile displays:
  - Header: provider glyph, title, status pill, `repo ⎇ branch`, model, live timer, cost/tokens.
  - Body: live terminal output view (monospace scrollback with ANSI-cleaned lines, plus
    last words & edited files for discovered sessions).
  - Footer: prompt input box (`Send to agent… ↩`, or `Queue when done ⇧↩`), plus an inline
    `1 Allow` / `2 Deny` bar whenever a permission or trust request is pending.
- `⌘⇧N` (or `+ New Session` button) opens the New Session modal: choose Agent, Folder,
  optional new Git worktree, and initial prompt.
