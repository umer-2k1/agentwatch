# Decision log

Append-only. A decision is never edited — if it changes, add a new entry that supersedes it.

| ID | Decision | Status |
|----|----------|--------|
| D-001 | Watch, never own the agent | active |
| D-002 | macOS first, Windows later, behind a `Platform` trait | active |
| D-003 | All agents supported, each behind an adapter + verification badge | active |
| D-004 | Both surfaces: ambient dots **and** a full window, one shared state | active |
| D-005 | Never acts by default; opt-in user-written rules only | active |
| D-006 | Collisions warn, never block | active |
| D-007 | Free now; paid tier later; watching stays free, acting is paid | active |
| D-008 | Target solo developers in v1 | active |
| D-009 | Process liveness ships in v0.1, not later | active |
| D-010 | Tauri (Rust + system webview), not Electron | active |
| D-011 | Append-only span log; derive aggregates, never pre-aggregate | active |
| D-012 | Only transcripts touched inside an active window are shown (default 12h) | active |
| D-013 | An open turn with no live process goes `unknown`, never `working` | active |

---

## D-001 — Watch, never own the agent

**Context.** Two architectures exist: read what other apps run (AgentDock), or launch the
agents yourself and own their I/O (Agentbox).

**Decision.** We watch.

**Why.** Owning the agent means rebuilding how the user runs agents — a new runner, a new
mental model, and an instant fight with the user's existing terminal setup. Watching means
the user changes nothing.

**Cost we accept.** We cannot detect a crash from files alone (mitigated by D-009), and we
cannot answer inside desktop applications ever. Both are surfaced honestly in the UI rather
than hidden.

**Revisit if.** We add an optional "own it" mode later — additive, never a replacement.

## D-002 — macOS first, Windows later

**Decision.** v1 is macOS only. Every OS-specific capability sits behind a `Platform` trait
so Windows is a second implementation, not a rewrite.

**Why.** Hooks, AppleScript and terminal control are all different on Windows. Splitting
effort now would ship both badly.

## D-003 — All agents, with an honest badge

**Decision.** Every agent AgentDock supports, we support. Each is one file implementing a
`Provider` trait. Each carries `verified` (tested against a live session) or `unverified`
(written from docs only).

**Why.** Coverage is table stakes; the badge is the honesty AgentDock buries behind the word
"preview".

**Cost.** 10+ undocumented formats to maintain. Mitigated by recorded fixtures (SPEC-002).

## D-004 — Two surfaces, one state

**Decision.** Ambient dot strip and full window are views over the same store. Presence is a
setting: dots on/off, window at launch, tray-only.

**Why.** Two surfaces driven by two code paths drift and feel like two products.

## D-005 — Never acts by default

**Decision.** The policy engine ships **empty**. No rule, no action. Users may write rules
(allow `npm test`, allow reads under `src/`), each logged and undoable.

**Why.** Auto-approval is the difference between a dashboard and a guardrail. Shipping
defaults means shipping someone else's risk appetite.

## D-006 — Collisions warn, never block

**Decision.** Same file touched by two agents inside 20 minutes ⇒ red ring, names both
agents, optional notification. Nothing is blocked. The set is also exposed as a queryable
API so agents can ask before editing.

**Why.** Blocking requires owning the process (D-001 forbids it).

## D-007 — Free now, paid later

**Decision.** Ship free. Later: watching free forever, acting (inbox answering, rules) paid.

**Why.** A coloured dot is not worth a paywall, but it is worth an audience.

## D-008 — Solo developers in v1

**Evidence.** Competitor traction is thin and entirely individual: 62 upvotes, 0 reviews,
one outside comment (a Windows user, who could not try it). Every feature is about one
person's attention. There is no proven team demand.

## D-009 — Liveness in v0.1

**Decision.** Ship the PID check with the first status view, not later.

**Why.** Without it, `working` / `idle` / `crashed` / `finished` are indistinguishable —
the single most common way these dashboards lie. It is ~20 lines.

## D-010 — Tauri, not Electron

**Why.** ~10–15 MB and ~40–60 MB RAM versus Electron's ~150–250 MB. The product's whole
promise is "always running in the background", which Electron makes expensive.

**Trade.** System webview means relying on WKWebView behaviour; acceptable for a
macOS-first product.

## D-011 — Spans, not summaries

**Decision.** Persist every state transition as an immutable row. Compute weekly/monthly
aggregates at read time.

**Why.** Pre-aggregating into daily buckets (what AgentDock does) makes questions like
"which weekday do I get interrupted most" permanently unanswerable.

## D-012 — Active window, not all history

**Context.** The first live scan found **246** transcripts, because Claude Code keeps every
session ever run for a project. Showing 246 dots is not a status board; it also took 11s to
parse them all on the first tick.

**Decision.** A transcript is a candidate session only if its mtime is inside
`active_window` (default 12 h, override with `AGENTWATCH_ACTIVE_WINDOW_HOURS`). Directory
scans are cached for 2 s.

**Consequence.** After the window, a session disappears even if nobody reviewed it. Accepted:
the board answers "what is happening now", not "what happened in March". History (M0.4)
will read the same files on demand, so nothing is lost.

## D-013 — `unknown` is a real state

**Context.** The same scan showed months-old abandoned transcripts as `working`, because
`turnClosed == false` was read as "in progress".

**Decision.** An open turn with no live process and no activity for `staleTurnAfter`
(default 15 min) resolves to `unknown`, with evidence *"turn never closed, no process,
quiet for X"*.

**Why.** Two lies are available here: call it `working` (it isn't) or call it `idle` (we
don't know). Both destroy trust in every other dot. `unknown` is the only answer that stays
true, and the evidence string tells the user exactly what we saw.
