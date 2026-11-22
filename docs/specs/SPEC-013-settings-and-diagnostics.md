# SPEC-013 — 7-Tab Settings, 30+ Agents Matrix, 13 Hosts & Live Diagnostics

Status: **specified** (Blueprint §4.7, §4.8, §9, §16.51, Figures 03–21, 41–49)

## 1. Settings Architecture (7 Tabs)

Settings matches AgentDock's 7-tab toolbar layout with icon-above-label tabs:

1. **General**:
   - Strip: Position (`Right edge`, `Left edge`, `By the Dock`, `Around the notch`, `Only in Menu Bar`),
     `Show in the Dock`, `Clicking an agent that needs you` (`Opens its card in the Inbox` / `Goes to the agent`),
     `Always show agent names`, `Fold idle agents after` (`5m`, `15m`, `30m`, `1h`, `Never`).
   - Shortcuts: `Ask with ⌥⌥`, `Inbox ⌃⌥I`, `Hover card ⌃⌥H`.
   - App: `Launch at login`, `Check for updates automatically`, `Welcome Tour…`.
2. **Inbox**:
   - Decision pop-up: `Pop up questions` (`Off`, `Center of the screen`, `Top of the screen`),
     `Keyboard goes to it`.
   - When an agent finishes: `Notify` (`Off`, `After every turn`, `Turns over 1/2/5/10 min`),
     `Style`, `Not when you're looking at it`, `Flash where the dots are`, `Bounce the Dock icon`,
     `Play a sound`, `Recaps and next steps`.
   - Also tell you: `Two agents change the same file`, `Summary when you come back`.
   - **Policy Engine (§16.43)**: user-defined auto-allow rules (`npm test`, `cargo test`, `git status`),
     empty by default (D-005).
3. **Appearance**:
   - Split view: controls on the left, **Live Preview** on the right (showing the Hover Card
     with subagent trees, collision banners, Today's token usage, and the folded dot strip!).
   - Look: `Appearance` (`System`, `Dark`, `Light`), `Background` (`Glass ↔ Solid` slider),
     `Rows` (`Tiles`, `List`, `Compact`), `Provider icons`.
   - Hover card field toggles: `Model`, `Turn timer and last activity`, `Tokens written and context`,
     `Git branch`, `Group by repository`, `Subagents`, `Today's usage` (`Hidden`, `Total only`, `Chart`).
4. **Agents**:
   - Left pane: 30+ agents grouped into *Supported* (`Claude Code`, `Codex`, `OpenCode`, `Grok`),
     *In preview* (`Cursor`, `Gemini CLI`, `GitHub Copilot`, `Kilo Code`, `Claude Cowork`,
     `Antigravity`, `Qwen Code`, `Factory Droid`, `Goose`, `Pi`, `Cline`, `Roo Code`, `Hermes`),
     and *In cmux and herdr* (13 more), with live green dot when detected on disk.
   - Right pane: icon, detected binary/store path badge, description, 4-row capability matrix,
     and live toggles (`Show sessions`, `Install Hook` with byte-identical backup/restore,
     `Register MCP server` in `~/.claude.json`).
5. **Apps & Diagnostics (§16.51)**:
   - Left pane: 13 host apps (`cmux`, `tmux`, `iTerm`, `WezTerm`, `kitty`, `herdr`, `Terminal`,
     `Ghostty`, `Warp`, `Alacritty`, `VS Code, Cursor, Zed`, `Claude app`, `Codex app`).
   - Right pane: live capability-probed status badge (fixes AgentDock Bug #2 by probing `cmux`
     subcommands and OpenCode SQLite schema version v1/v2!), 4-row capability matrix, setup
     instructions, and `Copy Diagnostics`.
6. **iPhone**:
   - Toggle `Answer from your iPhone` → starts local companion HTTP server (`:47811`), renders
     scannable QR code SVG, `Copy Link`, `New Link` (rotates bearer token), plus E2E Relay config.
7. **Tour & Roadmap**:
   - Interactive 5-step **Welcome Tour** (`41-tour-1` through `45-tour-5`) and in-app
     **Feature Requests & Roadmap** board.
