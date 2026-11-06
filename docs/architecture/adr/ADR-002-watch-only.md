# ADR-002 — Read-only over owning the agent

**Status:** accepted (see D-001)

## Context
Two ways to know an agent's state: read its files, or own its process.

## Decision
Read-only.

## Consequences
- **Gain:** the user changes nothing about how they work. No new runner to learn.
- **Lose:** no crash detection from files (fixed separately by ADC-003/ADR-003 liveness).
- **Lose:** no answering inside GUI apps, permanently and by construction.
- **Lose:** keystroke-based answering is fragile; the hook path is the robust one.
