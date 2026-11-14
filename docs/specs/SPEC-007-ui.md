# SPEC-007 — UI system

Status: **implemented** (session 7)
Related: D-017, D-018, D-019

## Stack

Vite · React 19 · TypeScript · Tailwind 3 · shadcn-style primitives (source-owned, in
`src/components/ui/`). No component library runtime we do not control.

## Surfaces

| Surface | Job | State |
|---|---|---|
| **Window** | read, answer, configure | shipped |
| **Menu bar** | always-visible counts + per-agent menu | shipped |
| **Edge overlay** | ambient dots over the desktop | M0.1b |
| **Settings** | appearance, live | shipped |

The window deliberately has **no dot rail** — that duplication was removed (D-018). Dots
belong where there is nothing else: the edge.

## Tokens

Everything visual is a CSS custom property in `src/index.css`.

| Group | Tokens |
|---|---|
| Surfaces | `--background` `--card` `--popover` `--border` `--muted` |
| Text | `--foreground` `--muted-foreground` |
| Action | `--primary` `--ring` (driveable by the accent picker) |
| Status | `--status-working` `--status-needs-you` `--status-done` `--status-idle` `--status-dead` `--status-unknown` |
| Brand | `--brand-claude` `--brand-codex` `--brand-opencode` `--brand-cursor` `--brand-gemini` `--brand-copilot` |
| Shape | `--radius` (live, 2–18 px) |

Two utilities do the heavy lifting:
- `.app-canvas` — the ambient warm/cool wash behind everything
- `.glass` — translucent panel + backdrop blur + saturation, so panels read as macOS chrome

## Colour rules

1. **Fill = status.** Never vendor, never decoration.
2. **Ring + chip = vendor**, via `--brand`. A vendor must survive in greyscale.
3. **Text always accompanies colour.** Every row has a status word; the board works for
   colour-blind users and in screenshots.
4. **`unknown` is a dashed hollow ring**, never a solid grey — "I don't know" must not look
   like "idle".

## Component rules

| Component | Rule |
|---|---|
| `SessionCard` | whole card is the click target; expands to status · why · folder · pid |
| Answer buttons | only rendered when a hook channel is open; absent means "cannot answer", never disabled-looking |
| `GroupSection` | ordered by the urgency of its most urgent member, not alphabetically |
| `Header` | counts in urgency order, one entry per present status |
| `SettingsSheet` | live preview, no Apply button |

## Keyboard

`1` allow · `2` deny · `↑`/`↓` move · `Esc` close. *(Not implemented yet — the backend
supports it; the frontend binds it in M0.3.)*

## Accessibility floor

- Every interactive element is a real `button` or has `role` + `tabIndex` + `Enter` handler.
- Contrast: light mode uses the light-appearance status palette (D-019) to stay ≥ 4.5:1.
- No information is conveyed by animation alone; the amber pulse is decoration, the word
  "needs you" is the signal.
