# ADR-001 — Tauri over Electron

**Status:** accepted (see D-010)

## Context
The app must run all day in the background while the user works. Memory cost is therefore a
product requirement, not a preference.

## Decision
Tauri 2 with a Rust core and a plain web frontend.

## Consequences
- Bundle ~10–15 MB, RAM ~40–60 MB. Electron is typically 150–250 MB.
- Rust is the natural home for the hot path: tailing files, `kill(0)`, parsing thousands of
  JSON lines per second.
- Cost: system webview (WKWebView) means we inherit its rendering quirks, and there is no
  Chromium to paper over cross-platform differences. Acceptable while macOS-first.
- Cost: Tauri's IPC is less convenient than Electron's `ipcMain`; we send whole snapshots
  rather than fine-grained deltas.
