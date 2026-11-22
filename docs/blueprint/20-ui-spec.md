# UI specification

## Surfaces (in priority order)

1. **Edge strip** — 28 pt wide borderless window, right edge, vertically centred, 12 pt dots
   in a dark pill. One dot per agent; colour = status; red ring = collision.
2. **Hover card** — opens on hover over the strip. Dark glass, 20 pt radius, 1 px border.
   Sections: optional error banner → "Now" + Mark all as seen → one row per agent
   (title · status · folder · branch · model · place · timer) → footer.
3. **Menu bar** — native NSMenu: Ask ⌥⌥ · Open Inbox ⌃⌥I · Activity… · live status rows ·
   Settings… ⌘, · Refresh ⌘R · Tour · Updates · trial/licence · Quit.
4. **Inbox panel** — floating; tabs Inbox | History; card anatomy below; key legend in footer.
5. **Settings** — 7 tabs: General · Inbox · Appearance · Agents · Apps · iPhone · License.
6. **Activity** — heatmap, day lanes, replay, leverage/waits/models/records, share card.

## Colour tokens (doc §14)

| Token | Hex |
|---|---|
| working | `#3B82F6` |
| needs-you | `#F59E0B` |
| done | `#22C55E` |
| idle | `#6B7280` |
| error | `#EF4444` |
| collision ring | `#EF4444` at 60 % |

Typography: SF Pro 13/14 pt body; SF Mono for paths and timers.

## Card anatomy (Inbox)

```
● Title                                    folder ⎇ branch · 2m · Claude app ↗
  BODY HEADER (permission type, uppercase)
  ┌ command / question block ───────────────────────────┐
  1  Yes
  2  No
  footer: 1–9 answer · J K move · E clear · ↩ go · esc close
```

Permission card shows the tool description as an uppercase header, the command in a mono
block, "Do you want to proceed?", then numbered options.

## Keyboard

| Key | Action |
|---|---|
| `1`–`9` | pick option |
| `J` / `K` | move |
| `R` | reply |
| `E` | mark seen |
| `↩` | answer / go to agent |
| `Tab` | Inbox ↔ History |
| `esc` | undo / close |

**Undo:** every answer waits **2 s** with a visible countdown bar; `esc` cancels.

## Appearance controls

Appearance (System/Dark/Light) · Background (Glass ↔ Solid) · Rows (Tiles/List/Compact) ·
Provider icons · hover-card field toggles (Model, Timer, Tokens, Branch, Group by repo,
Subagents, Today's usage).
