# AgentWatch — agent instructions

Read this before touching anything. It is short on purpose.

## Rules

1. **pnpm only.** Never `npm`, `npx`, or `yarn`. The lockfile is `pnpm-lock.yaml`.
   `pnpm install` · `pnpm tauri dev` · `pnpm tauri build`.
2. **Read `docs/HANDOFF.md` first.** It says exactly where the work stopped.
3. **Spec before code.** New behaviour gets a file in `docs/specs/` first.
4. **Every decision goes in `docs/03-decisions.md`** with its reasoning. Append, never edit.
5. **Commit small and often.** Conventional prefixes. Verify the build before you commit.
6. **Attribution matters:** `git config user.email` must be non-empty, or the contribution
   graph silently breaks. If it is empty, stop and fix it.
7. **Verify, don't assume.** Run the app. Run the tests. Read the screenshot.
