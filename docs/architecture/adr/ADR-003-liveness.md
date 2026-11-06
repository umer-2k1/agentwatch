# ADR-003 — Process liveness via PID

**Status:** accepted (see D-009)

## Context
Files cannot distinguish *finished* from *crashed* from *killed* — a dead agent writes no
goodbye. Competitors infer "idle" from timestamps and therefore lie.

## Decision
Record the agent's PID and check it with `kill(pid, 0)`.

## How the PID is obtained
- v0.1: process scan. For each session's `cwd`, look for a `claude` process whose working
  directory matches, and take the most recent.
- v0.2: authoritative. The hook process is a child of the agent, so `getppid()` inside our
  hook binary yields the agent's PID exactly.

## Guard against PID reuse
Store `(pid, start_time)`. A pid that now belongs to a different start time is not our agent.

## Consequences
- `dead` becomes a real state, which no competitor has.
- Costs one syscall per session every 2 s — negligible.
