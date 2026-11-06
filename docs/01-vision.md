# Vision

## One line

AgentWatch is a macOS status board for people who run several AI coding agents at once.

## The problem

Running 5–15 coding agents in parallel is now normal. Claude Code in one tab, Codex in
another, OpenCode in a third. The work isn't the problem — **knowing where they are** is.

Two things break at that scale:

1. **You can't see them.** An agent blocks on a permission prompt in a tab you aren't
   looking at, and you lose 20 minutes without knowing.
2. **You can't remember them.** At the end of a day you have no idea how many times you
   were interrupted, how long agents sat waiting on you, or which model did the work.

Every vendor builds a status view for *their own* agent. None of them will show you the
others — Anthropic will never display OpenAI's sessions. That gap is the product.

## The user

**v1: a solo developer on a Mac running 3–15 agents across 2+ vendors.**

They already have a terminal setup they like. They do not want a new IDE, a new editor, or
a new way to launch agents. They want to keep working exactly as they do now, and stop
losing track.

## What AgentWatch is

- A **reader**. It reads the session files agents already write. It does not run them.
- A **status board** — every agent as a dot on the edge of the screen.
- An **inbox** — every question and permission prompt in one place, answerable by keystroke.
- A **record** — a log of what ran, what waited, and how long you took to answer.
- **Yours** — local only, no account, no telemetry, fully themable.

## What AgentWatch is not

- Not an agent runner. It does not spawn, own, or supervise processes it did not start.
- Not an IDE, not an editor, not a replacement terminal.
- Not a scheduler or a queue. It does not decide what works next.
- Not cross-machine in v1. No sync, no cloud, no account.

## The honest limits (accepted, and shown in the UI)

| Limit | Cause | What we do |
|---|---|---|
| Cannot answer inside desktop apps | no terminal to type into, no hook contract | show it, let you jump to it |
| Cannot see permission prompts without a hook | prompts are never written to disk | hook install is the onboarding step |
| Cannot stop an agent | we don't own the process | warn only (decision D-006) |
| Some agents are read-only | no hook, or undocumented format | honest `verified` / `unverified` badge |

## Positioning

**The neutral shelf.** Every vendor shows only their own agent. AgentWatch shows all of
them, side by side, because it is not owned by any of them. That is the one thing a vendor
structurally cannot copy without advertising for a competitor.
