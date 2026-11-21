# AgentDock — captured reference (real app, on the user's machine)

Captured 2026-11-21 by driving the installed app with synthetic mouse events.
Screenshots sit beside this file. This is the **target**, not our design.

## 1. The menu bar is the primary surface

`01-hover-card.png` · `02-menu-bar-menu.png`

- Tray icon: a **vertical capsule with three stacked dots** (amber/blue/green), ~12×22 pt.
- Clicking it opens a **native macOS menu** (not a custom popover). Items, in order:

| Item | Shortcut |
|---|---|
| Ask an Agent… | ⌥⌥ |
| Open Inbox | ⌃⌥I |
| Activity… | — |
| Allow ⌥⌥ in Accessibility… | — |
| *(separator)* | |
| live status: `cmux: <error>` / `No agents running` (disabled rows) | |
| *(separator)* | |
| Settings… | ⌘, |
| Refresh | ⌘R |
| *(separator)* | |
| Welcome Tour… | |
| Request a Feature… | |
| Check for Updates… | |
| *(separator)* | |
| `Trial: 7 days left` (disabled) | |
| Buy AgentDock… | |
| Enter License… | |
| `AgentDock 1.5.5` (disabled) | |
| Quit AgentDock | ⌘Q |

**Takeaway:** the app is usable entirely from the menu. No window is required.

## 2. The edge dot and hover card

`05-edge-dot.png` · `01-hover-card.png`

- One dot per agent, on the **right screen edge**, vertically centred.
- Rendered as a coloured dot inside a dark circular pill (the pill is the hover target).
- Hover opens a card ~460 pt wide with:
  - a red **error banner** (icon + bold title, then a monospace detail block)
  - a **status section**: icon + bold headline (“No agents running”)
  - a body sentence in muted grey
  - a footer row: `Trial: 7 days left` … `Buy AgentDock` (blue link)
- Corner radius ~20 pt, dark blur, 1 px border.

## 3. Settings — 7 tabs

`03-settings-general.png` · `04-settings-agents.png`

Toolbar tabs, icon above label: **General · Inbox · Appearance · Agents · Apps · iPhone · License**.

**General** is grouped into titled sections, each a rounded card of rows:

- *Strip*: Position (Right edge ▾) · Show in the Dock (toggle + description) ·
  Clicking an agent that needs you (▾) · Always show agent names (toggle) ·
  Fold idle agents after (15 minutes ▾)
- *Shortcuts*: Ask with ⌥⌥ (✓ Allowed) · Inbox (⌃⌥I)
- *AgentDock*: Launch at login (toggle) · Check for updates automatically (toggle)

Row anatomy: **label left, control right**, optional description line under the label in
muted grey, hairline separators between rows, 14 pt labels.

**Agents** tab is a two-pane layout:

- Left: "Supported" (Claude Code, Codex, Grok) and "In preview" (14 more), each with an icon
  and a **green/grey status dot** for installed/not.
- Right: agent detail — big icon, name, "Not installed" badge, description, then a
  **capability table** with ✓ / ✗ / – and a qualifier column, then per-agent settings
  (e.g. "Claude Code hook" toggle), then "Where it runs" + **Set Up Apps…** button.

## What this means for AgentWatch

| Gap | Action |
|---|---|
| Menu bar menu | we have a menu; needs the same item set and disabled-status rows |
| Edge dot + hover card | we have no edge overlay at all |
| Settings window | we have a 4-control drawer; needs 7 tabbed panes |
| Per-agent detail | missing entirely |
| Capability table | missing |

Priority order when we resume: **edge dot + hover card → settings window → menu parity**.
