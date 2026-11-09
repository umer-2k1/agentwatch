# UI tour

Every screenshot here is the **real frontend** (`src/index.html`, `styles.css`, `main.js`)
rendered in headless Chrome with a mocked Tauri bridge. No design mockups — what you see is
what ships.

Regenerate: serve `src/` and append the harness, then

```bash
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new \
  --virtual-time-budget=1500 --window-size=980,620 \
  --screenshot=docs/assets/ui-board.png "http://localhost:8899/index.html?state=full"
```

## 1 — The board (`ui-board.png`)

The default view. A dot per agent, most urgent first.

- **Ring = vendor** (Claude clay, Codex teal, OpenCode amber, Gemini violet, Cursor grey)
- **Fill = status** (blue working, amber needs you, green done, red dead, dashed unknown)
- Footer states each provider and its verification badge

## 2 — Answering (`ui-answering.png`)

Three sessions need attention. Only the **Claude Code** row offers `1 Allow` / `2 Deny`.

That asymmetry is the product being honest: OpenCode has no hook, so its blocked row can be
seen but not answered. We show what we can do, and we do not pretend to do the rest.

## 3 — Light appearance (`ui-light.png`)

Same board, light palette. Status colours switch to Apple's *light* system colours —
`#0a84ff` on white is ~2.1:1 contrast, which is unreadable, so light mode uses `#0071e3`,
`#a85600`, `#1a7f37`, `#d70015`.

## 4 — Compact + pill + purple (`ui-compact-pill.png`)

The same data with `shape=pill`, `density=compact`, `accent=#bf5af2`. One attribute each;
no layout is duplicated per theme.

## 5 — Appearance drawer (`ui-appearance.png`)

Five controls, live preview, no Apply button.

## 6 — Empty (`ui-empty.png`)

No agents: the strip collapses and the board says what to start. An idle app should look
calm, not broken.

---

## Rules the UI follows

1. **Status is never colour-only.** Every row carries a text pill.
2. **Vendor never overrides status.** Brand is a ring and a chip, never the fill (D-014).
3. **Unknown is visible.** A dashed ring and a `?` — never silently grey.
4. **Evidence is always available.** Every row's tooltip is the resolver's `evidence.detail`.
5. **Nothing is hidden for tidiness.** If we cannot answer a session, the row says so by
   omission of the buttons, not by looking identical to one we can.
