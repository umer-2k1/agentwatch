# Acceptance tests (doc §18, adapted)

Each one is a test we must be able to run.

| # | Given | Then |
|---|---|---|
| 52 | a Claude session is running | `UserPromptSubmit` → blue within 1 s, timer runs; `Stop` → green + History row |
| 53 | Claude requests Bash permission | card shows the command; `1` shows a 2 s undo bar; `esc` cancels; after 2 s the hook returns allow and the command runs |
| 55 | 2 Claude + 2 OpenCode sessions | 4 distinct dots; Activity "at once" peak = 4 |
| 56 | two agents edit one file within 20 min | both dots get a red ring; `who_edits` lists both |
| 57 | a turn ends with "Login expired" | status `error:auth`, never "Finished", card offers login |
| 58 | unknown OpenCode schema | visible "unsupported version" banner, never silence |
| 60 | hook install → uninstall | `settings.json` is byte-identical |
| 62 | ⌥⌥ with no typeable agent | ↩ explains why nothing was sent |
| 64 | 20 idle agents | < 1 % CPU, < 150 MB RAM |

## Our own regression suite (already green)

- `dead` proven end-to-end (`tests/liveness_live.rs`)
- hook pipeline through the real binary (`tests/hook_pipeline.rs`)
- 63 Rust unit tests
