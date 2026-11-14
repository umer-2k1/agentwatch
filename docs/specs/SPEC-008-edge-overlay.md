# SPEC-008 — Edge overlay (M0.1b)

Status: **not implemented**

## What it is

A borderless, always-on-top, **non-activating** strip pinned to a screen edge, showing one
dot per agent. This is the surface the product is named for; the window is where you go when
a dot needs explaining.

## Hard requirements

| # | Requirement | Why |
|---|---|---|
| 1 | **Never steals focus** | an always-on-top window that takes keyboard focus makes the machine unusable. This is the single most common failure of overlay utilities |
| 2 | Click-through where there is nothing to click | the strip is 40 px wide; it must not eat clicks meant for the app underneath |
| 3 | Survives every Space / full-screen app | `NSWindowCollectionBehavior.canJoinAllSpaces + .stationary` |
| 4 | Follows the active display | repositions on `NSApplication.didChangeScreenParametersNotification` |
| 5 | Click a dot → open the window focused on that agent | the overlay is an index, not a console |
| 6 | Hover a dot → a hover card, without stealing focus | `NSPanel` with `becomesKeyOnlyIfNeeded` |
| 7 | Hides on request | a menu-bar toggle, remembered |

## Geometry

```
┌─────────────────────────────────────────────┐
│  screen                                     │
│                                    ▢  dot   │  ← 28 px wide gutter
│                                    ▢  dot   │     centred vertically
│                                    ▢  dot   │     8 px gap, 12 px dots
└─────────────────────────────────────────────┘
```

Position choices (a setting): right edge · left edge · above the Dock · around the notch
(macOS 14+) · menu-bar only.

## Implementation notes

- Tauri: second window, `decorations: false`, `transparent: true`, `alwaysOnTop: true`,
  `skipTaskbar: true`, `focus: false`, `resizable: false`.
- macOS specifics need `objc2` or a small `NSWindow` subclass:
  `NSWindowStyleMask.nonactivatingPanel`, `ignoresMouseEvents` toggled per-hit-region.
- The overlay and the window must share state. They read the same `SharedStore`; the overlay
  renders a **reduced** snapshot (status + repo only) to keep its payload tiny.

## Definition of done

- [ ] Dots visible over a full-screen video on a second display
- [ ] Typing in another app is never interrupted by the overlay
- [ ] Clicking a dot raises the window with that agent expanded
- [ ] CPU < 1% idle with the overlay visible
- [ ] Reducing the overlay to the menu bar is one click and persists

## Why this is worth the risk

The dots are the product's identity, and every alternative (a window you must open) is a
worse version of a dashboard. But if requirement 1 is not met, the feature must not ship —
a focus-stealing overlay is worse than none.
