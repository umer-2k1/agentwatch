# SPEC-009 — Inbox, History, and 2-Second Undo

Status: **specified** (Blueprint §4.4, §7, §7.1, §14.32)
Related: SPEC-004, D-005

## 1. Card Types in the Inbox (`⌃⌥I`)

Every Inbox item is a structured `InboxCard` derived from either a live hook event, an
embedded PTY prompt, an error state, or a finished turn:

| Card Type | Trigger | Answerable? | Actions |
|---|---|---|---|
| `permission` | `PermissionRequest` hook (Bash, Edit, Write, Network) or PTY permission prompt | ✅ Yes | Numbered options `1 Yes` / `2 No` / `3 Always allow in this repo`, diff/command preview, risk badge |
| `question` | `PermissionRequest` with `tool_name == "AskUserQuestion"` | ✅ Yes | Numbered options `1..9` with label + subtext, plus free-text answer |
| `plan` | Plan approval request | ✅ Yes | `1 Approve plan` / `2 Revise…` |
| `trust` | Folder-trust prompt (`Do you trust the files in this folder?`) | ✅ Yes (PTY) / Jump | `1 Yes, I trust this folder` / `2 No` |
| `auth` | Turn ended with `Login expired` or `error:auth` | ✅ Action | `Open login` (`claude /login`) — never shown as "Finished." (fixes AgentDock Bug #3) |
| `balance` | Turn ended with `Insufficient balance` (`error:balance`) | ✅ Action | `Switch model` |
| `elsewhere` | Session waiting in a non-typeable GUI app (`Claude app`, etc.) without a blocking hook | ❌ No | `"Waiting for you in <place>. Press ↩ to open it there."` |
| `finished` | Turn completed cleanly (`done`, unseen) | ✅ Follow-up | Agent's last words, `R` inline reply (`Reply to <title>…`), `V` cross-vendor review (`Ask Codex/OpenCode to review`), `E` mark seen |

## 2. The 2-Second Undo Window (Blueprint §4.4, §7.1, Figures 54–56)

When the user presses a digit `1..9` or clicks an option on an answerable card:

1. The card enters `pending_undo` state for `2000 ms` (`undo_deadline = now + 2000`).
2. The chosen option row highlights in blue (`#3B82F6`).
3. A bottom bar appears inside the card:
   `Sending "1. Yes"                                     esc undo`
   with a horizontal blue countdown progress bar shrinking from 100% to 0% over 2 seconds.
4. If the user presses `Esc` or clicks `undo` before the deadline:
   - The pending answer is cancelled (`undo_answer` command).
   - The card returns to `open` state immediately; nothing is sent to the hook or PTY.
5. When the 2-second timer expires without cancellation:
   - The decision is committed to the blocking hook (`hookSpecificOutput`) or PTY stdin.
   - The card transitions to `sent` and is removed from the Inbox; the badge decrements.

## 3. History Tab (`Tab` in Inbox)

- Filter pills: `All` | `Not seen · N` (toggled with `U`).
- Search input (`/` or `⌘F`): filters by every word typed across `title`, `last_words`,
  `folder`, `repo`, `branch`, `provider`, and `model`.
- Grouped by calendar day (`TODAY`, `YESTERDAY`, date) with a `Replay day ↗` link that opens
  Activity on that day.
- Each turn row displays: timestamp (`HH:MM`), status dot (`green` if ok, `red` if error),
  title, provider badge (`claude`, `codex`, `opencode`), `Not seen` badge, duration,
  `repo ⎇ branch · place`, last words (or error reason), `Now: Done, not seen yet`, and an
  inline reply input.

## 4. Keyboard Contract

`1–9` answer · `J / K` move selection · `R` focus reply · `V` ask review · `E` clear/mark
seen · `⇧E` mark all seen · `U` toggle unseen filter · `Tab` switch Inbox/History · `↩` go
to agent · `esc` undo or close.
