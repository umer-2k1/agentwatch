# SPEC-003 — UI surfaces (v0.1)

Status: **partial** — board shipped, edge overlay pending
Related: D-004

## Principle

**One state, two surfaces** (D-004). The frontend never computes status; it renders a
snapshot the resolver already produced. If a dot is wrong, the bug is in `resolver.rs`,
never in `main.js`.

## Surface A — the board (shipped)

A normal window: header counts, dot strip, list, footer.

| Element | Rule |
|---|---|
| Header counts | one entry per status present, urgency order |
| Dot strip | one dot per session, colour = status, label = title |
| Dot for `unknown` | dashed ring, never a filled grey dot — must not read as `idle` |
| Dot for `needsYou` | slow pulse, the only animated dot |
| List row | title, repo · branch · model, age, pid |
| Status pill | `unknown` gets a `?` suffix |
| Footer | provider verification badge (`verified` / `unverified`) |

## Surface B — the edge overlay (M0.1b, not shipped)

Borderless, always-on-top, **non-activating** window pinned to a screen edge. Must never
steal focus. Click-through where there is nothing to click. This is the surface the product
is named for; the board is the place you go when a dot needs explaining.

## Colour channels  (D-014)

| Channel | Encodes | Rule |
|---|---|---|
| dot fill | **status** | never used for anything else |
| dot ring, chip, brand tick | **vendor** | `--brand-*` tokens, one per agent |
| dashed hollow ring | **unknown** | must not read as a solid `idle` dot |

Status must also be legible without colour: every row carries a text pill.

## Theming readiness

Every colour, radius, gap and dot size is a CSS custom property in `:root`. M0.3 swaps that
block and nothing else. No hard-coded colour may appear outside `:root`.

## Evidence rule

Every dot and row carries the resolver's `evidence.detail` as a tooltip. A user who
disagrees with a dot must be able to see *why* it says that.
