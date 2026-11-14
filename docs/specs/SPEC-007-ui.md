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

---

## The gap that mattered (recorded after a failed attempt)

First implementation used **one bordered card per agent**. The user's reaction was that it
looked like a prototype. Comparing against AgentDock directly, the difference was structural,
not cosmetic:

| | Bordered cards (wrong) | One panel (right) |
|---|---|---|
| Container | one box per agent | **one** glass surface for the whole list |
| Separation | 6 px gaps | **hairlines** (`border-white/[0.045]`) |
| Grouping | group title above boxes | group title **inside** the panel |
| Attention | full amber border on the card | **2 px accent bar** at the left edge + 6 % tint |
| Status | grey pill on the right | **coloured word inline** with the meta |
| Vendor | coloured square | **glyph** (asterisk / hexagon / braces) |
| Rhythm | each card floats | rows share a baseline grid |

Rules derived from this:

1. **A list is one object.** Cards are for independent items (search results); a live board is
   a list, and a list is a single surface.
2. **Attention is shown at the edge**, never by drawing a full border — the accent bar reads
   without shouting.
3. **Status is a word with a colour**, not a coloured chip with a word. The word is the
   signal; colour reinforces it and survives nothing else.
4. **Meta is one line with hairline separators** (`needs you · shop-web · feature/pricing ·
   claude-opus-5`), not stacked labels.
