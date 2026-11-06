# AgentWatch — documentation index

This folder is the **project diary**. Anything a new engineer (or a new AI agent) needs to
pick this up cold lives here. Code changes; this folder explains *why*.

## Read in this order

| # | File | What it answers |
|---|------|-----------------|
| 1 | `01-vision.md` | What are we building, for whom, and what is it *not* |
| 2 | `02-scope.md` | What is in and out of each version |
| 3 | `03-decisions.md` | Every locked decision, with the reasoning |
| 4 | `architecture/ARCHITECTURE.md` | How the system is put together |
| 5 | `architecture/adr/` | One file per irreversible-ish choice |
| 6 | `specs/` | Behaviour specs — the contract before code |
| 7 | `milestones/` | What "done" means for each phase |
| 8 | `HANDOFF.md` | Where we are right now, and what to do next |

## Rules for maintaining this folder

1. **A spec exists before the code that implements it.**
2. **Every decision that is hard to reverse gets an entry in `03-decisions.md`.**
   Reversing one later means adding a new entry that supersedes it — never editing history.
3. **`HANDOFF.md` is updated at the end of every working session.** If it is stale, the
   session is not finished.
4. **Never delete a decision.** Mark it superseded and link to its replacement.
5. Docs are written for a reader who has none of this conversation's context.
