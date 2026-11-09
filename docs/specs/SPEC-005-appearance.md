# SPEC-005 — Appearance

Status: **shipped (client-side); persistence moves to the Rust config in M0.3b**
Related: D-014

## Principle

**Appearance is data, not code.** Every visual decision is a CSS custom property on `:root`.
Changing a theme writes one attribute and no component re-renders. This is what makes
"the user can change anything" cheap instead of a rewrite per setting.

## Controls (shipped)

| Control | Values | Affects |
|---|---|---|
| Appearance | Dark · Light · System | the whole surface palette |
| Accent | 7 swatches (blue, orange, green, purple, pink, cyan, white) | brand mark, selected control, focus ring |
| Dot shape | Circle · Pill · Square | `--dot-radius`, `--dot-w` |
| Density | Cozy · Compact | row padding, dot size |
| Glass | 0 → 30 px | backdrop blur only |

Live preview: no Apply button. Preferences persist in `localStorage` under
`agentwatch.prefs`.

## Rules

1. **Status colours are never user-editable.** Amber must always mean "needs you". A theme
   that could make `done` look like `needsYou` would break the only thing the board promises.
2. **Accent never touches status.** It is chrome: the mark, the selected segment, focus.
3. **Every surface is derived from a token.** No hard-coded colour outside `:root` and the
   `[data-theme]` blocks.
4. **Light appearance must keep contrast.** Status pills carry a text label regardless, so
   colour-blind and greyscale users are unaffected (D-014).

## Not yet built

- Dot position (edge / notch / dock) — belongs with the edge overlay (M0.1b).
- Theme export/import as JSON.
- Typography controls (UI font, mono font, scale).
- Moving `prefs` from `localStorage` into `config.json` so it survives a reinstall and can
  be shared between the board and the overlay window.
