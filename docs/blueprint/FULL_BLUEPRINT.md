AgentDock: Functional Analysis & Development Blueprint
What this is. A reverse-engineering-style analysis of AgentDock 1.5.5 (build 222), com.tihomirvidenov.AgentDock, done on 22 Nov 2026 on macOS 26 by driving the installed app, running real Claude Code and OpenCode sessions in parallel, calling its MCP server and its iPhone web API, and reading its bundle. It ends with a build plan for a new app with equivalent functionality, better usability and the features AgentDock lacks.
>
How to read the evidence tags. Every claim carries a tag:
[V] Verified: observed by running it (screenshot, API response or file diff).
[S] From the app's own strings: text compiled into the binary (UI copy, SQL, setting keys). Strong evidence of intent, but not exercised.
[I] Inferred: my reading of how it probably works internally. Treat as a design suggestion, not fact.
[B] Blocked: could not test; the blocker is named.
>
Screenshots live in screenshots/ next to this file and are referenced by filename.
Contents
1. Executive summary
2. Test environment, method and coverage checklist
3. Feature inventory
4. Feature-by-feature reference
5. User journeys (step-by-step)
6. Parallel Claude Code & OpenCode session management
7. User input, questions and permission handling
8. Observable architecture and integration contracts
9. Bugs, limitations and edge cases found
10. Functional & technical requirements for the new app
11. Suggested architecture
12. Recommended technology stack
13. Data models
14. UI/UX design specification
15. Error handling and edge cases
16. Improvement recommendations (beyond AgentDock)
17. Roadmap and priorities
18. Testing & acceptance criteria
19. Appendix: raw interfaces (MCP schema, iPhone API, hook config, file paths)
1. Executive summary
AgentDock is a passive, menu-bar "air-traffic control" for AI coding agents on a Mac. It does not run agents or host terminals. It *watches* agents you start elsewhere (any terminal, cmux, tmux, iTerm, WezTerm, kitty, editors, the Claude and Codex desktop apps) and gives you:
* The Strip: one coloured dot per agent on the screen edge. Blue = working, amber = needs you, green = done (unseen), grey = idle. [V] 25b-edge-strip-three-states.jpg
* Hover card: hover the strip to see every agent's title, status, folder, branch, model, turn timer, subagent tree and collision warnings. [V] 25-hover-card-needs-you-cmux-error.jpg
* Inbox (⌃⌥I): a keyboard-driven queue of questions, permission prompts and finished turns, answerable with a keystroke and a 2-second undo. [V] opened, answered a permission (with undo) and a question through the real hook contract (§7.1). 53–57
* Ask (⌥⌥): a Spotlight-style box to type a message into any agent's terminal, or start a new one. [V] opened. [B] sending (no typeable agent). 39-ask-panel-no-agents.jpg
* Activity: a GitHub-style 12-month heatmap, per-day swim-lanes with replay, leverage, wait-time, model mix, records and a shareable image. [V] 29…37-*.jpg
* MCP server: lets agents query each other ("who else is editing this file?") and message or start agents. [V] all 6 tools called.
* iPhone page: a token-protected LAN web page mirroring agents and the Inbox, plus a relay for remote access. [V] 22-iphone-web-page.jpg
Integration model: it reads each agent's own on-disk session files (zero setup). It adds optional hooks (Claude Code ~/.claude/settings.json, Codex ~/.codex/hooks.json) for real-time question and permission events. It types into terminals through each multiplexer's CLI or AppleScript (cmux, tmux, iTerm, WezTerm, kitty, herdr).
Business model [V]: 7-day trial, then $19 one-off for up to 3 Macs (Stripe). After the trial, status dots and the hover card keep working; Inbox answering, ⌥⌥ and iPhone are paywalled. Sparkle auto-update and Setapp distribution are also supported. 20-settings-license.jpg
The big opportunity for your app: AgentDock is a *monitor* that depends on other apps to host sessions, and it breaks when those apps change (two compatibility bugs found today, §9). Your app can own the session lifecycle: spawn Claude Code and OpenCode in embedded PTYs, so you control input, permissions and recovery directly. Keep AgentDock's best ideas: zero-setup discovery, the edge strip, the keyboard-first Inbox with undo, Activity analytics and the MCP coordination server.
  

Figure: Edge strip with amber/blue/green dots  (25b-edge-strip-three-states.jpg)
  

Figure: Hover card: cmux error banner, Needs you / working timer / Done  (25-hover-card-needs-you-cmux-error.jpg)
  

Figure: Answerable permission card (1 Yes / 2 No) via the hook  (53-inbox-permission-card-answerable.jpg)
  

Figure: "Sending '1. Yes' · esc undo" 2-second bar  (54-inbox-answer-sending-undo-bar.jpg)
  

Figure: After Esc: answer undone, card back  (55-inbox-after-esc-undo.jpg)
  

Figure: After answering No: card gone, badge decremented  (56-inbox-after-answer-sent.jpg)
  

Figure: AskUserQuestion card with numbered options  (57-inbox-question-card.jpg)
  

Figure: ⌥⌥ panel: "No agents to message"; Return does nothing  (39-ask-panel-no-agents.jpg)
  

Figure: Activity: 12-month heatmap + today's lanes  (29-activity-top.jpg)
  

Figure: Leverage and Waiting-on-you cards  (30-activity-leverage-waiting.jpg)
  

Figure: Models, Subagents, "That's like…", Records  (31-activity-models-thatslike-records.jpg)
  

Figure: Models, Subagents, "That's like…", Records  (32-activity-records.jpg)
  

Figure: A busy day: 9 session lanes, "4 at once" peak  (33-activity-busy-day-lanes.jpg)
  

Figure: Replay cursor mid-day ("1 working · 5 turns done")  (34-activity-replay-running.jpg)
  

Figure: Month and Week hour grids + model split  (35-activity-month-grid.jpg)
  

Figure: Month and Week hour grids + model split  (36-activity-week.jpg)
  

Figure: Share image (period, look, copy/save)  (37-share-activity-card.jpg)
  

Figure: The served phone page (Inbox + Agents, live timer)  (22-iphone-web-page.jpg)
  

Figure: Trial, Buy $19, license key  (20-settings-license.jpg)
2. Test environment, method and coverage checklist
Environment. macOS 26 (Darwin 25.4), MacBook Pro, 1389×868-pt display. AgentDock 1.5.5 in trial (7 days left). Claude Code CLI ~/.local/bin/claude (login expired), OpenCode CLI v2.0.18 (~/.opencode/bin/opencode), OpenCode desktop app, cmux app, Claude desktop app.
Method.
20. Bundle inspection: Info.plist, embedded binaries (AgentDock, agentdock-hook, agentdock-mcp, AgentDockWidgets.appex, Sparkle.framework) and ~14k extracted strings.
21. GUI driven with computer use: background mode for windows, full-screen mode for hover, the menu bar and global hotkeys.
22. Real sessions in cmux: 2× Claude Code CLI, 2× OpenCode TUI, 1× agent started *by AgentDock's own MCP start_agent*, plus 3 Claude desktop sessions.
23. MCP server driven directly over stdio JSON-RPC.
24. iPhone web page opened in a browser at phone size and its JSON API read.
25. File-system diffs of everything AgentDock writes.
Coverage checklist
#
	Feature
	Status
	Evidence / blocker
	1
	Edge strip dots (4 states)
	✅ Tested
	25b, 51 (done→idle after "mark seen")
	2
	Hover card
	✅ Tested
	25, 50, 51
	3
	Menu-bar menu
	✅ Tested
	38-menubar-menu.jpg
	4
	Dock icon badge/menu
	⚠️ Partial
	Setting seen; right-click needs a Finder grant (not requested)
	5
	Inbox: open, cards, keys legend
	✅ Tested
	26, 27
	6
	Inbox: answer a Claude CLI question or permission
	✅ Tested (hook contract) · ⚠️ real CLI sessions not held
	§7.1; 52–57
	7
	Inbox: Claude-app question
	✅ Tested
	Card type elsewhere, answerable:false, "Press ↩ to open it there"
	8
	History tab
	✅ Tested
	28-history-tab.jpg
	9
	Ask ⌥⌥ panel
	✅ Opened · ⛔ send
	"No agents to message" (no typeable agent; cmux read broken)
	10
	Activity (heatmap, day/week/month, replay, records)
	✅ Tested
	29–36
	11
	Share activity image
	✅ Tested (not saved)
	37
	12
	Settings: General
	✅ Captured
	03
	13
	Settings: Inbox
	✅ Captured
	05, 06 (popup values from strings)
	14
	Settings: Appearance + live preview
	✅ Captured
	07, 08, 46, 47
	15
	Settings: Agents (30+ agents)
	✅ Captured
	09–13
	16
	Claude Code hook install/uninstall
	✅ Tested
	Diffed ~/.claude/settings.json; restored byte-identical
	17
	MCP registration toggle
	✅ Tested
	Added and removed agentdock in ~/.claude.json
	18
	OpenCode session reading
	❌ Fails
	OpenCode 2.x schema mismatch (§9 bug 1)
	19
	Settings: Apps (13 hosts)
	✅ Captured
	14–18
	20
	cmux integration
	❌ Fails to read · ✅ creates workspaces
	"Unknown command 'current'" (§9 bug 2)
	21
	Settings: iPhone (QR, link, relay)
	✅ Tested
	19, 21, 22; API mapped
	22
	License / trial
	✅ Viewed
	20; purchase not attempted (payment)
	23
	Welcome Tour (5 steps)
	✅ Tested
	41–45
	24
	Request a Feature / Roadmap
	✅ Viewed
	48, 49; nothing submitted (it posts publicly)
	25
	MCP: list_agents/get_agent/who_edits/recent_turns
	✅ Tested
	§19
	26
	MCP: send_message
	✅ Tested (refusal path)
	Claude-app agents refused with a clear error
	27
	MCP: start_agent
	✅ Tested
	Created cmux workspace 10 running Claude
	28
	File-edit tracking & collisions
	✅ Tested
	edited_files in ~5 s; same_checkout collision; 50
	29
	Notifications, sound, Dock bounce, flash
	⚠️ Not triggered
	Options documented; "Notify" was Off
	30
	Decision pop-up
	⚠️ Not triggered
	Off by default; needs an answerable request
	31
	Recaps & next steps (Apple Intelligence)
	⛔ Blocked
	"Needs Apple Intelligence, which is off"
	32
	Codex hook / Codex desktop answering
	⚠️ Not tested
	Out of scope (Claude Code + OpenCode focus)
	33
	Widgets (Activity, Tokens)
	⚠️ Inspected only
	WidgetKit appex present; not added to desktop
	34
	Check for updates (Sparkle)
	⚠️ Not clicked
	Feed URL in Info.plist
	35
	Summary when you come back
	⚠️ Not triggered
	Needs 15+ min away
	

  

Figure: Done → Idle after Mark all as seen  (51-after-mark-all-seen.jpg)
  

Figure: Collision lines "Same checkout as …"  (50-hover-card-same-checkout-collision.jpg)
  

Figure: Menu-bar menu with error row and agent rows  (38-menubar-menu.jpg)
  

Figure: Inbox: "elsewhere" card + finished card with Reply and Ask Codex to review; key legend  (26-inbox-open-with-hover.jpg)
  

Figure: Inbox: "elsewhere" card + finished card with Reply and Ask Codex to review; key legend  (27-inbox-cards-zoom.jpg)
  

Figure: Real CLI sessions: "Waiting for you in Terminal" (not answerable)  (52-inbox-cli-cards-not-answerable.jpg)
  

Figure: History: All/Not seen, search, Replay day, failed turns shown as "Finished."  (28-history-tab.jpg)
  

Figure: Settings › General: strip, shortcuts, launch/update  (03-settings-general.jpg)
  

Figure: Settings › Inbox: decision pop-up, Codex desktop, notify  (05-settings-inbox-top.jpg)
  

Figure: Settings › Inbox: flash/bounce/sound/recaps, collisions, come-back summary  (06-settings-inbox-bottom.jpg)
  

Figure: Appearance with the live demo card + strip preview  (07-settings-appearance-preview.jpg)
  

Figure: Hover-card field toggles, Today's usage  (08-settings-appearance-hovercard.jpg)
  

Figure: Row styles  (46-rows-tiles-preview.jpg)
  

Figure: Row styles  (47-rows-compact-preview.jpg)
  

Figure: Agent detail: "Not installed", capability matrix  (09-settings-agents-grok.jpg)
  

Figure: Claude Code: installed path, sessions/hook/MCP toggles (before)  (10-settings-agents-claude-code.jpg)
  

Figure: OpenCode: preview, "Not yet" answering/typing  (11-settings-agents-opencode.jpg)
  

Figure: Codex: hook + MCP toggles  (12-settings-agents-codex.jpg)
  

Figure: Full agent list incl. "In cmux and herdr"  (13-settings-agents-list-bottom.jpg)
  

Figure: cmux setup steps (Automation socket mode)  (14-settings-apps-cmux.jpg)
  

Figure: Terminal.app: why typing isn't possible, Ask Now…  (15-settings-apps-terminal.jpg)
  

Figure: VS Code / Cursor / Zed capabilities  (16-settings-apps-editors.jpg)
  

Figure: Claude app capabilities  (17-settings-apps-claude-app.jpg)
  

Figure: tmux: TMUX_PANE, Copy Install Command  (18-settings-apps-tmux.jpg)
  

Figure: iPhone off; on with Safari QR, Copy/New Link, relay + pairing QR  (19-settings-iphone-off.jpg)
  

Figure: iPhone off; on with Safari QR, Copy/New Link, relay + pairing QR  (21-settings-iphone-on-qr.jpg)
  

Figure: Welcome Tour steps 1–5  (41-tour-1-welcome.jpg)
  

Figure: Welcome Tour steps 1–5  (42-tour-2-where-agents-run.jpg)
  

Figure: Welcome Tour steps 1–5  (43-tour-3-dots-on-edge.jpg)
  

Figure: Welcome Tour steps 1–5  (44-tour-4-answer-anywhere.jpg)
  

Figure: Welcome Tour steps 1–5  (45-tour-5-make-it-yours.jpg)
  

Figure: Featureloop requests and roadmap  (48-feature-requests.jpg)
  

Figure: Featureloop requests and roadmap  (49-feature-roadmap.jpg)
3. Feature inventory
Area
	Features
	Ambient status
	Edge strip (position, fold idle, names, provider icons, red collision ring); hover card; menu-bar icon with needs-you badge; Dock badge & menu; desktop widgets
	Act
	Inbox (questions, permissions, plans, finished turns, reply, follow-ups, "Ask Codex to review", undo); decision pop-up; ⌥⌥ Ask/new agent; click-to-jump to the agent's tab or pane or app; iPhone page
	Observe & learn
	History (search, unseen filter, replay); Activity (heatmap, timelines, replay, leverage, waits, models, subagents, records, "that's like…" comparisons); share card; Tokens widget
	Coordinate
	MCP server: list/get agents, who_edits, recent turns, send_message (queued when_done), start_agent; file-collision detection
	Integrate
	30+ agents (3 supported, 14 preview, 13 "via cmux/herdr"); 13 hosts (cmux, tmux, iTerm, WezTerm, kitty, Ghostty, Terminal, Warp, Alacritty, VS Code/Cursor/Zed, Claude app, Codex app, herdr)
	Configure
	7-tab Settings; onboarding tour; launch at login; auto-update
	Commerce
	Trial, license key (3 Macs), Stripe checkout, deactivate, Setapp build
	Feedback
	In-app feature requests and roadmap (Featureloop SDK)
	

4. Feature-by-feature reference
4.1 The Strip (edge dots)
* Purpose. Peripheral-vision status for every agent, without opening anything.
* How it works [V]. A borderless always-on-top window (28 pt wide) on the right screen edge, vertically centred, holds one 10-pt dot per agent inside a dark pill. Colour = status. When an agent needs you, the menu-bar icon shows a count badge ("1").
* States [V]: working (blue) · needs_you (amber) · done (green, finished and unseen) · idle (grey, seen or quiet). "Mark all as seen" turns Done into Idle. 51-after-mark-all-seen.jpg
* Options [S]/[V] (03-settings-general.jpg):
* *Position*: Right edge (default), "Beside the agent strip", "Around the notch", "Only in the Dock".
* *Show in the Dock*: on = a Dock badge plus a Dock menu of agents, and AgentDock appears in ⌘-Tab.
* *Clicking an agent that needs you*: "Opens its card in the Inbox" (default) or "Goes to the agent". ⌘-click does the opposite.
* *Always show agent names*: labels next to the dots.
* *Fold idle agents after*: 5 / 15 (default) / 30 / 60 minutes. Folded agents collapse into "+N".
* *Provider icons* (Appearance): the dot shows the agent's logo (Claude, Codex, Gemini, Grok; a square for others).
* *Group by repository* (Appearance): worktrees of one repo sit together.
* Collision ring [S]. "Two agents change the same file → their dots get a red ring either way; this also sends a notification."
* Limitations. Dots are tiny (10 pt) and hard to hit. There is no keyboard path to the strip itself. The hover card only opens on a real mouse hover.
4.2 Hover card
* Purpose. Expand the strip into a readable list.
* Content [V] (25, 50): an optional error banner (e.g. "cmux not reachable" plus the raw error); a "Now" header with Mark all as seen; one row per agent with title, status word ("Needs you"/"Done"/"Idle"), folder, ⎇ branch, model, place ("Claude app", "Ghostty"…), a live turn timer (blue mm:ss) or age ("8m"), the subagent tree with per-subagent timers, and collision lines ("Same checkout as '…'"); a footer with trial status and "Buy AgentDock".
* Row styles [V]: Tiles (default), List, Compact. 46-rows-tiles-preview.jpg, 47-rows-compact-preview.jpg
* Toggleable fields [V] (08): Model, Turn timer & last activity, Tokens written & context, Git branch, Group by repository, Subagents, Today's usage (Hidden/…).
* Look [V]: Appearance System/Dark/Light; Background slider Glass↔Solid.
* Data note [S]: "Model, tokens and usage come from your agents' session files on this Mac, read while the card is open and every 10 minutes for Activity and the Tokens widget."
4.3 Menu-bar menu
38-menubar-menu.jpg [V]. Items: Ask an Agent… ⌥⌥ · Open Inbox ⌃⌥I · Activity… · (when not granted) *Allow ⌥⌥ in Accessibility…* · live error row (e.g. cmux: Error: Unknown command 'current'…) · one row per agent with a coloured status dot · Settings… ⌘, · Refresh ⌘R · Welcome Tour… · Request a Feature… · Check for Updates… · Trial: 7 days left · Buy AgentDock… · Enter License… · AgentDock 1.5.5 · Quit ⌘Q. It is a native NSMenu, not a popover.
4.4 Inbox (⌃⌥I) and History
* Purpose. One queue for everything that needs a human, answered from the keyboard.
* Layout [V] (27-inbox-cards-zoom.jpg): a floating panel with Inbox (count badge) | History tabs, a "move to side" button and the ⌃⌥I hint. Each card has a status dot, title, folder, branch, age and place chip ("Claude app ↗"), then the body:
* *Question / permission* card: numbered options. Press 1–9 to choose, or click [S].
* *Elsewhere* card [V]: "Waiting for you in Claude app. Press ↩ to open it there." Not answerable.
* *Finished* card [V]: the agent's last message, a Reply field (R), Ask Codex to review (V), and optional AI "next steps" chips [S].
* *Plan approval* [S]: "Approve this plan?" Multi-permission [S]: "Grant these permissions for this turn?", "Allow for this session", "Allow for this turn".
* Keys [V]: J/K move · E clear (mark seen) · ↩ answer in the app or go to the agent · esc close · R reply · Tab switches Inbox/History · digits answer [S].
* Undo [S]: "Every answer waits two seconds… esc undoes." "One answer at a time."
* Safety [S]: "That agent has unsent text in its prompt, so nothing was sent. Clear or send it there first."
* History tab [V] (28): filters All / Not seen, search ("Every word has to be in the turn: title, last words, folder, branch, agent or model"), and grouping by day with a Replay day ↗ link. Each turn shows time, title, "Not seen" badge, duration, folder, branch, place, last words, "Now: Done, not seen yet" and a reply field. Keys: U not-seen-only, E seen, ⇧E all, ↩ go to agent, R reply.
* Decision pop-up [S] (05): *Pop up questions* (Off / Top of the screen / Center of the screen) shows one request until answered. *Keyboard goes to it* ("After ⌃⌥I or a click" vs "A second after it appears").
4.5 Ask an Agent (⌥⌥)
39, 40 [V]. A Spotlight-style panel opened by double-tapping Option (needs Accessibility permission). It has "Ask an agent…" input, a To: picker (↑/↓), the hints ↩ send · ⌥↩ new line · esc close, and the syntax "name: picks · new: starts one". The message goes to the most active agent by default, or to the one named by a prefix ("web: run the tests") [S]. Messages to a busy agent queue and are "Typed when the agent finishes" [S]. Observed: with no typeable agents it shows "No agents to message", and ↩ does nothing, with no feedback (§9).
  

Figure: ⌥⌥ panel: "No agents to message"; Return does nothing  (40-ask-panel-enter-no-feedback.jpg)
4.6 Activity window
[V] 29–37.
* Header: "The last 12 months: 81 hours of agent work · 1,621 turns in 184 sessions · 43 of 365 days · 191M tokens · ≈ $114 at API prices · streak · top model."
* Heatmap: 53×7 squares, toggle *Agent hours / Turns*. Click a day to open it; ←/→ step days.
* Day view: a "You — at your Mac" lane (away periods shaded, where away = sleep, lock or 5 min idle), an "At once" concurrency line with a peak marker ("4 at once · 2:33 PM"), then one swim-lane per session showing turn spans. Replay animates a cursor through the day, showing "1:24 PM · 1 working · 5 turns done" (34). Week and Month views are hour×day grids with an "Average day" histogram (35, 36).
* Cards: Leverage (agent-hours per hour at the Mac, here vs away) · Waiting on you (total wait, count, longest, median, buckets <1 / 1–5 / 5–15 / 15–60 / >60 min) · Models (share by hours) · Subagents · "That's like…" (fun comparisons: "15M tokens = War and Peace 19 times") · Records: longest turn, most at once, busiest day, best streak, night owl, your rhythm. Each record has "Replay the day ↗".
* Share [V] (37): a 1080×1080 image with period 7 days/30 days/12 months and look Night/Day. "Totals only: no titles, repositories, branches." Actions: Copy Image, Save…, Done.
* Backfill [S]: "Reading your past sessions for the days before AgentDock 1.1. This takes a minute, once."
4.7 Settings (7 tabs)
Tab
	Contents (verified unless marked)
	General (03)
	Strip (position, Dock, click behaviour, names, fold) · Shortcuts (⌥⌥ "Allowed", Inbox ⌃⌥I) · Launch at login · Auto-update · Version + Check Now · Request a Feature · Terms/Privacy
	Inbox (05,06)
	Decision pop-up (pop-up position, keyboard focus) · Codex desktop: Answer from the Inbox (preview) · When an agent finishes: Notify (Off / After every turn / Turns over 1, 2, 5, 10 min), Style (macOS notification "Top right; follows Focus…" / "Banner from the top center" / "Out of the notch on a MacBook; ignores Focus"), Not when you're looking at it, Flash where the dots are, Bounce the Dock icon, Play a sound, Recaps and next steps (Apple Intelligence) · Also tell you: Two agents change the same file, Summary when you come back (After 10 / 15 / 30 min / an hour away)
	Appearance (07,08)
	Look (Appearance, Background glass↔solid, Rows, Provider icons) · Hover card fields · live preview pane with a demo strip and card
	Agents (09–13)
	Left list: *Supported* (Claude Code, Codex, Grok), *In preview* (Cursor, Gemini CLI, GitHub Copilot, OpenCode, Kilo Code, Claude Cowork, Antigravity, Qwen Code, Factory Droid, Goose, Pi, Cline, Roo Code, Hermes), *In cmux and herdr* (Kimi Code, Amp, Crush, Junie, Kiro, Copilot Chat, Windsurf, Zed, Warp, OpenHands, Mistral Vibe, Aider, DeepSeek TUI). A green dot means installed. Right pane: icon, install badge with the detected path, description, a capability matrix (See what it's doing / History, Activity and Tokens / Answer questions from the Inbox / Type replies and ⌥⌥ messages, each ✓, ⊖ "Not yet" or "With Claude Code's hook"), per-agent toggles (Show sessions, Hook, MCP), and "Where it runs → Set Up Apps…". Footer: Show agents in preview, Show herdr's agents
	Apps (14–18)
	13 hosts, each with a status badge ("cmux isn't running", "Not installed", "Turn on the Claude Code hook…"), a capability matrix (see / answer / type / click-to-go), numbered Set up steps and action buttons (Open cmux, Copy Install Command, Ask Now…, Open Automation Settings)
	iPhone (19,21)
	Toggle "Answer from your iPhone" → Safari QR + LAN IP + Copy Link / New Link (rotates the token; "Paired phones stop working until they scan the new code") · iPhone app: Relay URL (optional, E2E-encrypted) + pairing QR
	License (20)
	Free trial days left · "AgentDock for Mac — Buy for $19" (Stripe, 3 Macs) · License key field + Activate + "Find my key" · Deactivate on This Mac [S]
	

4.8 Welcome Tour (5 steps)
41–45 [V]: (1) Welcome: the strip, the Inbox, Ask ⌥⌥. (2) Where your agents run: a live check of cmux with Open cmux / Check Again (it showed our real cmux error). (3) Dots on the edge: legend of the 4 states and click rules. (4) Answer from anywhere: key-cap illustrations and a live "⌥⌥ is allowed" check. (5) Make it yours: Appearance, Background, Rows, Launch at login, with a live preview. Then "Start Using AgentDock". There is also a terms gate: "Agree and Get Started" [S].
4.9 MCP server (agentdock-mcp)
[V] A stdio MCP server, a thin client that forwards to the app over the Unix socket ~/.agentdock/mcp.sock. It returns "AgentDock isn't running. Open it from /Applications." when the app is down [S]. Registered on demand with claude mcp add / codex mcp add. There are 6 tools; the full schemas are in §19. Verified behaviours:
* list_agents returns id (claude:<sessionId>), name, kind, status, folder, place, model, turn start, last activity and queued count.
* get_agent adds branch, repository, edited_files (last 20 min), queued messages and a collision description.
* who_edits(path) returns agents with their files and collisions[{kind:"same_checkout"|…}]. Run it before editing.
* recent_turns returns title, folder, branch, place, duration, finished_at and last_words.
* send_message against a Claude-app agent fails with isError "AgentDock can't type into …; it runs in Claude app. Open it there."
* start_agent{kind:claude|codex, folder, prompt} → {"status":"started"} and a new cmux workspace really opened running Claude. With a missing folder: "No folder at …".
4.10 iPhone web page and relay
[V] 22-iphone-web-page.jpg. A plain-HTTP server on port 47811 (all interfaces). Without the token: 401 "Open the link from AgentDock › Settings › iPhone on this phone first." The link is http://<lan-ip>:47811/?t=<token>; after the first load the token is held by the page (no cookie observed). The page (9.8 KB inline JS) polls GET /api/state every 2 s (skipped while hidden) and re-renders timers every 500 ms. Actions are POST /api/<action> with undo, seen, choose{card,number}, step{card,number}, reply{card,text}, clear{card}. The state shape is in §19. The relay [S] passes messages "encrypted with a key that's in the pairing code, never sent to the relay".
4.11 Widgets (WidgetKit)
[S] AgentDockWidgets.appex: Activity ("How much your coding agents worked: today, the last 7 days, or the last 20 weeks"; records like best streak) and Tokens ("Tokens your coding agents read and wrote: today, the last 7 days, and by model"). Data is shared through the App Group 9Q6VR23WUE.com.tihomirvidenov.AgentDock.
4.12 Feature requests & roadmap
[V] 48, 49. Embedded Featureloop SDK: Requests (search, Newest, Filter by tags, Submit Request, votes with a 5-minute undo) and a Roadmap (In Progress / Up Next / shipped). It was empty in this install.
4.13 Licensing, trial, updates
[V]/[S]: the trial is stored in defaults (trial.startedAt, trial.registered) and in ~/Library/Application Support/AgentDock/trial. License errors are handled distinctly: invalid, expired, refunded/disputed, already used on another Mac, test key, server busy, unreachable. Sparkle appcast: https://raw.githubusercontent.com/tvidenov/AgentDock-releases/main/appcast.xml, checked daily. A Setapp variant exists, and two copies running at once are detected ("AgentDock from Setapp is running. Quit it first").
5. User journeys (step-by-step)
J1: First run. Launch → menu-bar icon appears (LSUIElement app) → Welcome Tour (5 steps; live cmux and Accessibility checks) → agree to terms → the strip appears on the right edge. Any already-running Claude/Codex sessions show up immediately from their session files. [V]
J2: See what's happening. Glance at the dots → hover them → the card lists agents with timers → click a working agent to jump to its terminal tab, pane or app. [V] hover; click-to-jump verified only as the "↩ open in Claude app" hint.
J3: An agent asks a question (CLI + hook). [S]/[B] The agent calls AskUserQuestion or needs a permission → Claude Code runs the PermissionRequest/Notification hook → agentdock-hook sends the event to ~/.agentdock/events.sock and blocks (timeout 3600 s) → the dot turns amber and the badge increments → optionally a pop-up or notification → user presses ⌃⌥I → digit → 2-second undo window → the hook returns hookSpecificOutput with the decision (allow/deny, or the chosen answer) → the agent continues. Declines return "The user declined this in AgentDock." [V] end to end through the hook (§7.1), with the caveat that real CLI sessions were not held.
J4: A Claude-app session asks. [V] The card says "Waiting for you in Claude app. Press ↩ to open it there." The user must answer in the Claude app.
J5: Agent finishes. [V] Dot → green; a History entry with last words; Inbox "finished" card with reply, follow-ups and "Ask Codex to review". E or "Mark all as seen" → grey.
J6: Message an agent from anywhere. [S] ⌥⌥ → type ("web: run the tests") → ↩. If the agent is busy, the message is queued and typed when its turn ends. If its prompt already has unsent text, nothing is sent and you are warned.
J7: Start a new agent. [V via MCP] ⌥⌥ "new: …" or MCP start_agent → a new cmux workspace in the folder, running the agent with the first prompt.
J8: Coordinate agents. [V] An agent calls who_edits("/path/app.py") before editing and sees other agents and collisions. Two agents in one checkout get a "Same checkout as …" line on both cards.
J9: Review your week. [V] Activity → heatmap → click a day → Replay → read Leverage and Waiting → Share image.
J10: Phone. [V] Settings › iPhone → on → scan QR or Copy Link → the phone page lists the Inbox and agents live; answer there with the same undo.
6. Parallel Claude Code & OpenCode session management
Important framing: AgentDock does not create or host sessions (except start_agent / ⌥⌥ new:, which ask cmux to create a workspace). Parallelism lives in the host (cmux/tmux/terminals). AgentDock *observes* and *routes input*.
What I ran [V]
#
	Where
	Agent
	Task
	Outcome
	1
	cmux ws "CC-fix-bug"
	Claude Code CLI
	fix add() and run python
	Folder-trust prompt (answered "Yes, I trust this folder"), then "Login expired · Please run /login"
	2
	cmux ws "CC-ask-question"
	Claude Code CLI
	AskUserQuestion then edit
	Same login block
	3
	cmux ws "OC-readme"
	OpenCode 2.0.18 TUI
	add Usage to README
	First model: "Insufficient balance". Switched via /models to an OpenCode Go model → completed in 6.1 s
	4
	cmux ws "OC-tests"
	OpenCode TUI
	write pytest tests and run
	Completed: "3 failed, 3 passed" (it found the planted bug)
	5
	cmux ws (created by AgentDock)
	Claude Code
	MCP start_agent "say hi"
	Workspace created by AgentDock; Claude blocked by login
	6
	Claude desktop
	Claude Code (Haiku)
	question → edit → Bash
	Status needs_you, card elsewhere
	7–8
	Claude desktop
	this analysis session + a side question
	n/a
	Tracked live: working → done, timers, model "Opus 5.5"
	

How AgentDock saw them
* Claude Code (CLI and desktop) [V]: discovered with no setup from ~/.claude/projects/**/<session>.jsonl. It keeps a byte offset per file (history.json → scan.files[path].offset) and tails new lines. It extracts title, model, first prompt, pending words, "helper" flag and inCodexApp. Failed CLI turns still appeared in History, but labelled "Finished." (§9 bug 3). Desktop sessions are labelled "Claude app" and are not typeable.
* OpenCode [V]: not shown at all during two full, successful runs, even with "Show OpenCode sessions" on. Root cause verified (§9 bug 1).
* Concurrency [V]: Activity's "At once" lane recorded "2 at once · 5:38 AM" from the simultaneous Claude starts.
* Switching [S]/[V]: click a dot, or ↩ on a card → focus the agent's host (cmux pane via the cmux CLI, a tmux pane via TMUX_PANE, iTerm tab via AppleScript tty matching, WezTerm via WEZTERM_PANE, kitty via remote control, Terminal via AppleScript tab switch).
* Termination [V]: closing a cmux workspace removes the agent from list_agents within seconds (the processes exit). Hook SessionEnd also reports it [S]. Idle agents fold after N minutes.
* Host detection [S]: the hook records TERM_PROGRAM, TERM_SESSION_ID, ITERM_SESSION_ID, TMUX_PANE, WEZTERM_PANE, KITTY_WINDOW_ID/KITTY_LISTEN_ON, HERDR_PANE_ID, CURSOR_PROJECT_DIR and CLAUDE_CODE_ENTRYPOINT, which is how it knows where to type and where to jump.
Typing support matrix [V from the Apps tab]
Host
	See
	Answer from Inbox
	Type replies / ⌥⌥
	Click to go
	cmux
	✓
	✓
	✓
	✓ (its pane)
	tmux
	✓
	✓
	✓ (send-keys)
	✓
	iTerm / WezTerm / kitty / herdr
	✓
	✓
	✓
	✓
	Terminal.app
	✓
	Claude, with hook
	✗ "can't take typed text… without pressing Return"
	✓ (AppleScript)
	Ghostty / Warp / Alacritty
	✓
	Claude, with hook
	✗
	brings app forward
	VS Code / Cursor / Zed
	✓
	Claude, with hook
	✗
	brings app forward
	Claude app / Codex app
	✓
	✗ (Claude) / preview (Codex)
	✗
	opens the session
	

7. User input, questions and permission handling
Event source. Claude Code hooks installed by AgentDock [V] (exact JSON in §19):
Hook event
	Timeout
	Role
	PermissionRequest
	3600 s
	Blocks the agent until answered in AgentDock (or timeout)
	Notification
	5 s
	"needs you" signals (idle prompt, etc.)
	UserPromptSubmit
	5 s
	Turn started (timer)
	PostToolUse
	5 s
	Activity, file edits
	Stop
	5 s
	Turn finished
	SessionStart / SessionEnd
	5 s
	Lifecycle
	

The hook binary (~/.agentdock/hook, copied from the bundle) reads the event JSON on stdin, adds host/terminal env info, sends it to ~/.agentdock/events.sock, and for permission requests waits for a reply. It writes Claude Code's hookSpecificOutput with behavior (allow/deny), answers and message, e.g. "Declined in AgentDock." [S] Before writing, it backs up settings.json to ~/.claude/settings.json.agentdock-backup [V]. Uninstall restores the file exactly [V: byte-identical diff].
Request types [S]: "Allow this command?", "Allow these file changes?", "Allow network access?", "Approve this plan?", "Grant these permissions for this turn?", multi-step questions (step{card,number}), free-text answers ("Write your answer"), and "Waiting for you in a form AgentDock can't read yet."
Answer UX [S]/[V]: digits pick options; R opens reply; each answer sits for 2 s with esc = undo; "One answer at a time". A changed or already-answered request shows "This Codex request changed or was already answered." Unsent text in the agent's prompt blocks typing.
OpenCode [V-UI]: "OpenCode doesn't record when it waits for a permission, so only its questions turn the dot amber. Answer it in its terminal." Not answerable from the Inbox.
Recommendation for your app: own the PTY, so every prompt (Claude Code permission dialogs, OpenCode permission prompts, y/n prompts) is detected from the session's event stream or the agent's SDK/server API (OpenCode exposes an HTTP server), not by scraping files. That removes the "answer it in its terminal" gaps.
7.1 Re-test after CLI login (22 Nov, 06:00–06:25) [V]
Setup. Claude CLI logged in (claude auth status → loggedIn: true). AgentDock's Claude Code hook was switched on again. Sessions were started three ways.
Where the session ran
	What AgentDock showed
	Answerable from the Inbox?
	cmux (CLI-permission, CLI-question)
	Not listed at all in list_agents
	✗ cmux starts Claude with --settings <tmp> that injects cmux's own hook set, including its own PermissionRequest handler (cmux hooks feed --source claude). The prompt is handled through cmux, and AgentDock must read cmux, which fails (bug 2).
	Plain PTY / terminal (Claude's own permission dialog and AskUserQuestion)
	Listed within seconds as needs_you, place "Terminal", amber dots, badge 3–4
	✗ Card: "Waiting for you in Terminal. Press ↩ to open it there." No AgentDock hook process was waiting, so the real requests were not held; Claude showed its native dialog. 52-inbox-cli-cards-not-answerable.jpg
	Hook contract tested directly (a PermissionRequest JSON for that same session id piped into ~/.agentdock/hook claude)
	An answerable card replaced the "elsewhere" card
	✅ Yes. See below.
	

Permission card (53-inbox-permission-card-answerable.jpg): it shows the tool description as an uppercase header ("PRINT A TEST STRING"), the command in a mono block, "Do you want to proceed?" and the options 1 Yes / 2 No. The footer changes to 1–9 answer · J K move · E clear · ⇧ all done · ↩ go to agent · esc close.
* Pressing 1 shows "Sending '1. Yes' · esc undo" with a 2-second progress bar (54). Esc cancels it and the card is back (55).
* Pressing 2 and waiting: the card disappears and the badge drops from 2 to 1 (56). The hook process, which had been blocked for 19.5 s, printed:
{"hookSpecificOutput":{"hookEventName":"PermissionRequest",
 "decision":{"behavior":"deny","message":"The user declined this in AgentDock."}}}
	

Question card (AskUserQuestion sent as a PermissionRequest with tool_name:"AskUserQuestion"; 57-inbox-question-card.jpg): it shows the header ("LANGUAGE"), the question, and numbered options with their descriptions. Pressing 3 made the hook print:
{"hookSpecificOutput":{"hookEventName":"PermissionRequest",
 "decision":{"behavior":"allow","updatedInput":{"questions":[…unchanged…],
   "answers":{"Which greeting language?":"French"}}}}}
	

So questions are answered by allowing the tool call and passing `updatedInput.answers` (question text → chosen label). This is the contract to copy.
Open question [I]. Why weren't the *real* CLI requests held? Two plausible causes: AgentDock decides from the payload or environment (e.g. no recognised terminal tty or pane for a raw PTY, or permission_mode) that it can't answer and returns immediately; or Claude Code 2.1.330 sends AskUserQuestion and permission dialogs through a path that doesn't wait on PermissionRequest the same way. To confirm, wrap the hook command with a logger that tees stdin. Implication for your app: don't rely only on the hook. When you own the PTY you can also detect and answer Claude's native dialog directly.
Other observations: ⌃⌥I sent as a synthetic keystroke did not open the Inbox (the global hotkey ignores synthesized events); the menu item "Open Inbox" did. Claude auto-runs read-only commands such as echo without asking, so permission tests need a write command (touch).
8. Observable architecture and integration contracts
                       ┌───────────────────────── AgentDock.app (Swift/SwiftUI, LSUIElement) ──────────────────────────┐
Claude Code ──hook──▶ │ events.sock ◀── agentdock-hook (CLI, stdin JSON)                                              │
(any terminal)        │                                                                                               │
session .jsonl ─tail─▶│ Adapters: Claude · Codex · OpenCode(sqlite) · Gemini · Copilot · Goose(sqlite) · Qwen ·        │
~/.codex, sqlite dbs  │           Cursor · Cline · Antigravity · Cowork · Hermes · Droid …                            │
                      │        │                                                                                      │
                      │        ▼                                                                                      │
                      │  Agent registry (status machine) ─▶ Strip/HoverCard/Menu/Dock/Inbox/History/Activity (SwiftUI)│
                      │        │            │                    │                                                    │
                      │        │            │                    └─▶ activity.json / history.json (App Support)       │
                      │        │            └─▶ Router: cmux CLI · tmux · iTerm AppleScript · WezTerm cli · kitty rc   │
                      │        └─▶ mcp.sock ◀── agentdock-mcp (stdio MCP) ◀── claude/codex "agentdock" MCP server      │
                      │  HTTP :47811 (iPhone page, token) · Relay (E2E) · Sparkle · Featureloop · Stripe license       │
                      └──────────────────────────────────────────────────────────────────────────────────────────────┘
	

Verified files and paths:
* ~/.agentdock/{hook, mcp, events.sock, mcp.sock} (sockets mode srw-------).
* ~/Library/Application Support/AgentDock/activity.json (v5): days[{key, lanes, spans, subagents}], subagentsThrough, backfilled. Grows to ~200 KB per year.
* …/history.json (v1): turns[], scan.files[path] = {offset, inCodexApp, firstPrompt, model, pendingWords, helper, name}.
* …/trial; defaults domain com.tihomirvidenov.AgentDock: settings.tab, settings.agent, strip.dockIcon, strip.quietAfterMinutes, strip.groupByRepository, inbox.codexDesktop, agents.experimental, activity.sharePeriod, onboarding.version, trial.*.
* The OpenCode adapter SQL (from strings) reads session, message, part filtered by directory IN (<cwd of running opencode processes>) → "while it runs".
* Process discovery [I]: scans running processes for known binaries (claude, codex, opencode, gemini, cursor-agent, kiro-cli, qwen-code, …) and maps cwd/tty to host panes.
* Apple Intelligence [S]: an on-device prompt writes the one-line recap and three follow-ups ("Never invent results").
9. Bugs, limitations and edge cases found
#
	Severity
	Finding
	Evidence
	1
	High
	OpenCode 2.x sessions are invisible. OpenCode 2.0.18 writes sessions to the new session_v2 / session_message tables (70 rows today). AgentDock queries only the legacy session/message/part tables (newest legacy row: May 2026). It labels OpenCode "In preview… until verified".
	[V] sqlite schema + two successful OC runs never listed
	2
	High
	cmux integration broken by a CLI rename. Error everywhere: "cmux: Error: Unknown command 'current'". Reading/typing fails ("No agents to message"), yet start_agent still creates workspaces. cmux has moved to cmux workspace … subcommands with legacy aliases, but current was not kept.
	[V] 25, 38, 42
	3
	Medium
	Failed turns are reported as success. Claude CLI turns that ended in "Login expired" appear in History as "Finished." with green dots. No auth or error state exists.
	[V] 28
	4
	Medium
	⌥⌥ Ask gives no feedback when there's no recipient: ↩ does nothing and nothing explains why.
	[V] 40
	5
	Medium
	Claude-app (desktop) prompts can't be answered from the Inbox or iPhone, even with the hook. answerable:false.
	[V]
	6
	Medium
	cmux needs a security downgrade. Setup requires Socket Control Mode = Automation ("allow external local automation clients… no ancestry check").
	[V] 14
	7
	Low
	iPhone page is plain HTTP on all interfaces, with a bearer token in the URL (stated in the UI).
	[V]
	8
	Low
	Feature Requests window loses its toolbar title on the Roadmap tab (titles "Roadmap" without traffic-light alignment).
	[V] 49
	9b
	High
	Real Claude CLI permission/question prompts were not held by the hook: sessions in cmux are routed to cmux's own hooks; in a plain terminal the cards showed "Waiting for you in Terminal" and Claude kept its native dialog. Only a directly injected hook payload produced an answerable card.
	[V] §7.1
	9
	Low
	Popup menus and hover are mouse-only; hover card and strip have no keyboard access.
	[V]
	10
	Info
	Folder-trust and login are not surfaced. A fresh CLI session sat on Claude's "trust this folder" prompt and AgentDock showed nothing actionable.
	[V]
	11
	Info
	OpenCode permission waits are not detected (by design, stated in UI).
	[V] 11
	12
	Info
	"Summary when you come back", Recaps (Apple Intelligence), the Codex desktop Inbox and widgets were not exercised.
	[B]
	

10. Functional & technical requirements (new app)
Functional (MUST = parity, SHOULD = improvement):
* F1 MUST: Discover Claude Code and OpenCode sessions with zero setup (session files / OpenCode server API), plus a pluggable adapter interface for others.
* F2 MUST: Spawn and host sessions in embedded PTY terminals (Claude Code, OpenCode, shell), N in parallel, each in its own git worktree when requested. *(AgentDock lacks this.)*
* F3 MUST: Unified status machine per agent: starting → working → needs_input(question | permission | plan | trust | auth) → done → idle, plus error(kind) and exited(code).
* F4 MUST: An Inbox with keyboard answers, a 2 s undo, a reply composer, a free-text answer and multi-question steps.
* F5 MUST: Answer permissions for both Claude Code (hook PermissionRequest + hookSpecificOutput) and OpenCode (its server's permission endpoint / SDK events).
* F6 MUST: Ambient status: menu-bar icon with badge, optional edge strip, hover card, notifications (configurable thresholds, quiet when focused).
* F7 MUST: Jump to the session (internal tab) or to the external host (cmux/tmux/iTerm…) for discovered sessions.
* F8 MUST: History and search of turns; Activity analytics (heatmap, day lanes, concurrency, waits, tokens and cost per model).
* F9 MUST: MCP server with list_agents, get_agent, who_edits, recent_turns, send_message(when_done), start_agent (for both claude and opencode), plus new request_lock / release_lock.
* F10 MUST: File-edit tracking and collision detection (same file in N minutes, same checkout), with suggested worktree isolation.
* F11 SHOULD: Mobile/remote companion over HTTPS + pairing, or an E2E relay.
* F12 SHOULD: Session recovery: persist sessions; on relaunch offer claude --resume <id> / opencode -s <id> per tab.
* F13 SHOULD: Error surfacing: auth expired, insufficient balance, model unavailable, trust prompt, rate limit. Each gets a red/amber state and a one-click fix.
Non-functional:
* N1: Status latency ≤ 1 s for hook events and ≤ 3 s for file-tail events.
* N2: Idle CPU < 1 %; memory < 150 MB with 20 agents.
* N3: Privacy: everything local; the network is used only for update checks, licensing and the optional relay.
* N4: Hook install is idempotent, backed up and fully reversible (byte-identical uninstall).
* N5: Adapters tolerate schema drift: probe the tables/columns and degrade with a visible "unsupported version" message instead of silently showing nothing (fixes bug 1).
* N6: Host CLI calls are version-probed (cmux capabilities) with feature flags (fixes bug 2).
* N7: Accessibility: every surface reachable by keyboard, with VoiceOver labels.
11. Suggested architecture
┌──────────────────────── Desktop app (UI process) ────────────────────────┐
│  Shell: menu-bar item · Main window (Sessions grid / Inbox / Activity)   │
│  Overlays: edge strip, hover card, decision pop-up, ⌥⌥ quick-send        │
└───────────────▲─────────────────────────────────────────▲────────────────┘
               │ typed IPC (events + commands)            │
┌───────────────┴───────────── Core daemon (Rust) ────────┴────────────────┐
│ SessionManager ── PTY pool (portable-pty) ── per-session ring buffer     │
│ Orchestrator: spawn(kind, folder, prompt, worktree?), stop, resume       │
│ Adapters: ClaudeCode {jsonl tail, hooks, resume} · OpenCode {HTTP server │
│   + SSE events, sqlite fallback w/ schema probe} · Codex · generic PTY   │
│ StatusEngine (state machine) · PromptDetector (hook + adapter events)    │
│ InboxService (cards, undo queue, answer routing)                         │
│ EditTracker (tool events + FSEvents) · CollisionDetector                 │
│ HostRouter (cmux/tmux/iTerm/WezTerm/kitty) w/ capability probing         │
│ Store (SQLite, WAL) · Analytics rollups · Notifier                       │
│ Servers: hook socket · MCP (stdio shim → socket) · optional HTTPS/relay  │
└──────────────────────────────────────────────────────────────────────────┘
	

Key decisions:
26. Daemon/UI split. The daemon keeps sessions alive across UI restarts (recovery), and the MCP shim and hook talk to it over a Unix socket (same pattern as AgentDock, which works).
27. Own the PTY for sessions you start; watch files for sessions you didn't. Both feed the same AgentEvent stream.
28. OpenCode via its server API first. opencode 2.x runs a background service, and the TUI is a client. Subscribe to its event stream for messages and permission requests, and answer through the API. Fall back to sqlite with schema probing.
29. Event-sourced store. Append AgentEvents and derive status, history and analytics from them. Replay and Activity then come for free.
12. Recommended technology stack
Layer
	Recommendation
	Why
	Core daemon
	Rust (tokio, portable-pty, notify/FSEvents, rusqlite, serde)
	Fast, safe PTY + file watching, small memory
	Desktop UI
	Tauri 2 + React + TypeScript + Tailwind, terminal via xterm.js (WebGL addon)
	Cross-platform; your existing ~/AgentWatch is already Tauri + Vite + Tailwind, so it can grow into this
	macOS niceties
	Small Swift plugin (Tauri plugin) for the menu-bar badge, edge-strip NSPanel, global hotkey (⌥⌥ needs a CGEventTap + Accessibility), notifications, WidgetKit
	Things web views can't do well
	Alternative (Mac-only parity)
	SwiftUI + AppKit + SwiftTerm
	What AgentDock uses; best native feel, Mac-only
	Storage
	SQLite (WAL) + JSON settings
	Queryable history/analytics
	MCP
	Rust MCP server over a Unix socket + a tiny stdio shim binary
	Same pattern as agentdock-mcp
	Mobile
	PWA over HTTPS (self-signed + pinned), or an E2E relay (Noise/X25519)
	Fixes the plain-HTTP limitation
	Updates & licensing
	Tauri updater / Sparkle; Stripe + Keygen/LemonSqueezy
	Proven
	Tests
	cargo test, Playwright (UI), fake-agent fixtures, golden JSONL/sqlite fixtures per agent version
	Prevents schema-drift regressions
	

13. Data models
-- Agents/sessions (one row per agent session, discovered or spawned)
CREATE TABLE session (
 id TEXT PRIMARY KEY,                 -- "claude:<uuid>" | "opencode:<ses_id>"
 kind TEXT NOT NULL,                  -- claude | opencode | codex | shell | ...
 origin TEXT NOT NULL,                -- spawned | discovered
 title TEXT, folder TEXT NOT NULL, repo TEXT, branch TEXT, worktree TEXT,
 host TEXT,                           -- internal | cmux | tmux | iterm | claude_app | ...
 host_ref TEXT,                       -- pane id / tty / workspace ref
 pid INTEGER, model TEXT, parent_id TEXT,   -- parent = subagent tree
 status TEXT NOT NULL,                -- starting|working|needs_input|done|idle|error|exited
 status_detail TEXT,                  -- question|permission|plan|trust|auth|balance|rate_limit
 turn_started_at INTEGER, last_activity_at INTEGER,
 seen_at INTEGER, created_at INTEGER, ended_at INTEGER, exit_code INTEGER,
 resume_cmd TEXT                      -- for recovery
);

CREATE TABLE turn (
 id TEXT PRIMARY KEY, session_id TEXT REFERENCES session(id),
 prompt TEXT, last_words TEXT, outcome TEXT,          -- ok | error | interrupted
 error_kind TEXT, started_at INTEGER, finished_at INTEGER,
 tokens_in INTEGER, tokens_out INTEGER, cache_read INTEGER, cache_write INTEGER,
 cost_usd REAL, model TEXT, seen INTEGER DEFAULT 0
);

CREATE TABLE request (                 -- Inbox cards needing a human
 id TEXT PRIMARY KEY, session_id TEXT, type TEXT,     -- question|permission|plan|trust|auth|free_text
 title TEXT, body TEXT, options_json TEXT, steps_json TEXT,
 answerable INTEGER, created_at INTEGER, answered_at INTEGER,
 answer_json TEXT, answered_via TEXT,                 -- inbox|popup|phone|terminal|mcp
 undo_deadline INTEGER, state TEXT                    -- open|pending_undo|sent|expired|superseded
);

CREATE TABLE message_queue (           -- send_message when_done
 id TEXT PRIMARY KEY, session_id TEXT, text TEXT, when_done INTEGER,
 created_at INTEGER, sent_at INTEGER, state TEXT
);

CREATE TABLE file_edit (
 session_id TEXT, path TEXT, at INTEGER, tool TEXT, PRIMARY KEY(session_id, path, at)
);

CREATE TABLE presence (                -- "You at your Mac" lane
 start INTEGER, end INTEGER, kind TEXT                -- here | away(sleep|lock|idle)
);

CREATE TABLE event (                   -- append-only source of truth
 seq INTEGER PRIMARY KEY AUTOINCREMENT, at INTEGER, session_id TEXT,
 type TEXT, payload_json TEXT
);
	

Settings (JSON): strip {position, showInDock, clickNeedsYou, showNames, foldAfterMin, providerIcons, groupByRepo}; inbox {popup, popupFocus, notify, notifyStyle, quietWhenFocused, flash, bounce, sound, recaps, collisions, comebackSummaryMin}; appearance {theme, glass, rows, hoverFields[]}; agents {<kind>: {show, hook, mcp}}; phone {enabled, token, relayUrl}.
14. UI/UX design specification
Design principles: ambient first, keyboard always, one queue for decisions, never lose a prompt, explain every failure.
30. Main window: "Mission Control" (new; AgentDock has none):
* Left rail: Sessions · Inbox (badge) · Activity · Settings.
* Sessions grid: 1/2/4/6-up tiles; each tile = header (agent logo, title, status pill, branch, model, timer, tokens/$) + live terminal (xterm.js) + footer (prompt box, Approve/Deny bar when a permission is pending). Click a tile to focus; ⌘1…9 switch; ⌘⇧N opens the new-session sheet (agent: Claude Code | OpenCode, folder, optional new worktree, model, permission mode, first prompt).
* Filter chips: All · Needs you · Working · Done · Errors · per-repo.
31. Edge strip + hover card: keep AgentDock's design (dark glass, 20 pt radius, 1 px border, dots 10 pt → raise to 12 pt with a 24 pt hit area). Add red for error, a ring for collisions, and a keyboard shortcut to open the card (⌃⌥H).
32. Inbox: keep the card anatomy and keys (J/K, digits, R, E, ↩, esc, 2 s undo bar with countdown). Add a diff preview for file-change permissions, the command preview with a risk badge for Bash, "Always allow this command in this repo", and batch actions ("Approve all read-only").
33. Quick-send (⌥⌥): recipient chips with status dots. Always show why sending is impossible ("No typeable agents: start one with new:").
34. Status colour tokens: working #3B82F6, needs-you #F59E0B, done #22C55E, idle #6B7280, error #EF4444, collision ring #EF4444 at 60 %. Typography: SF Pro 13/14 pt body, SF Mono for paths and timers.
35. Notifications: by type (question/permission urgent; finished batched). Suppressed when the session is focused. A "while you were away" digest.
36. Onboarding: 4 steps, each with a live check (agents detected; hook installed with diff preview; hotkey permission; first session launched). End by launching a demo session.
37. Responsive: main window min 900×600; tiles reflow; overlays respect the notch and Stage Manager.
15. Error handling and edge cases
Case
	Required behaviour
	Agent CLI logged out / token expired
	Detect from output or hook (Login expired) → status error:auth → card "Claude Code is logged out" + Open login (runs claude /login in a new tab)
	Provider billing/balance error (OpenCode "Insufficient balance")
	error:balance → card with Switch model (list models covered by the plan)
	Folder-trust prompt on first run
	needs_input:trust card with Trust / Don't trust
	Adapter schema drift (OpenCode session_v2)
	Probe the schema; if unknown → banner "OpenCode 2.x not supported yet" instead of silence; telemetry-free local log
	Host CLI changed (cmux current)
	Capability probe on start; per-feature fallback; clear banner with a "Copy diagnostics" button
	Two answers in flight
	Serialize; second waits; show queue
	Request superseded or answered in the terminal
	Mark the card superseded, animate it out, toast "answered in terminal"
	Hook process times out (agent left waiting)
	Daemon restart must re-attach pending requests (persist them); on timeout tell the user the agent fell back to its own prompt
	App quits while hooks installed
	Hook fails fast ("app not running"), returning *no decision* so the agent shows its native prompt (never blocks for 1 h)
	Unsent text in the target prompt
	Refuse to type; explain (AgentDock does this; keep it)
	Duplicate app copies (Setapp + direct)
	Single-instance lock with a clear message
	Many agents (20+)
	Fold idle; virtualized lists; per-repo grouping
	Sleep/wake
	Re-scan processes and session files; reconcile statuses; presence lane marks "away"
	

16. Improvement recommendations (beyond AgentDock)
38. Host the sessions (PTY grid) with worktree-per-agent isolation by default, which prevents the same_checkout collision AgentDock can only warn about.
39. First-class OpenCode: server-API integration with full Inbox answering of permissions and questions (AgentDock: "Not yet").
40. Error and auth states, not "Finished.", for failed turns.
41. Cost & budget guardrails: per-session and daily $/token caps with pause-on-limit; live cost on tiles.
42. Diff-aware approvals: show the patch inline; approve per hunk.
43. Policy engine: per-repo rules ("auto-allow npm test", "always ask for rm"), synced with Claude Code's own permissions.allow.
44. Agent-to-agent handoff: "Ask OpenCode to review Claude's diff" (generalising AgentDock's "Ask Codex to review").
45. File locks via MCP (request_lock(path)), on top of who_edits.
46. Session templates & fan-out: run the same prompt on Claude Code and OpenCode side by side and compare the results.
47. Recovery: restore the grid after a crash or reboot with resume commands.
48. Search across transcripts (FTS5) and "replay a session".
49. Secure mobile: HTTPS + pairing, push notifications, approve from the lock screen.
50. Keyboard everywhere + command palette (⌘K).
51. Diagnostics page: every integration with probe results (this would have shown bugs 1 and 2 instantly).
17. Roadmap and priorities
Phase
	Scope
	Exit criteria
	P0: Core (2–3 wks)
	Rust daemon, SQLite event store, Claude Code adapter (jsonl tail + hook install/uninstall with backup), PTY spawn of Claude Code and OpenCode, sessions grid with xterm.js, status machine
	4 parallel sessions (2 CC + 2 OC) run, statuses are correct within 1 s
	P1: Inbox (2 wks)
	Permission/question cards for CC (hook) and OC (server API), 2 s undo, reply, keyboard nav, notifications, menu-bar badge
	Approve a CC Bash permission and an OC permission from the Inbox; undo works
	P2: Ambient (1–2 wks)
	Edge strip, hover card, ⌥⌥ quick-send, click-to-focus, quiet-when-focused
	All AgentDock strip/hover behaviours reproduced; keyboard access added
	P3: Coordination (1–2 wks)
	Edit tracking, collisions, worktree-per-agent, MCP server (6 tools + locks)
	who_edits returns within 5 s of an edit; collisions shown on both cards
	P4: Insight (2 wks)
	History + FTS search, Activity (heatmap, day lanes + replay, waits, leverage, models, cost), share image
	Numbers match the raw event store in tests
	P5: Reach (2 wks)
	Discovery of external sessions (cmux/tmux/iTerm) via a capability-probed HostRouter, mobile PWA over HTTPS, error/auth states, diagnostics page
	External sessions visible and typeable in tmux and cmux; auth error card shown on an expired login
	P6: Polish
	Onboarding with live checks, widgets, policies, budgets, licensing/updates
	Fresh install to first answered permission in < 3 min
	

18. Testing & acceptance criteria
Fixtures. Golden transcripts per agent and version: Claude Code jsonl (normal, AskUserQuestion, permission, login-expired, subagents); OpenCode sqlite v1 and v2 schemas; a fake hook client; a fake cmux/tmux CLI that can simulate renamed commands.
Acceptance tests (Given/When/Then):
52. *Status*: Given a CC session running, when UserPromptSubmit fires, then its tile/dot turns blue within 1 s and the timer starts. When Stop fires, then it turns green and a History row exists.
53. *Permission*: Given CC requests Bash permission, then a card with the command appears; pressing 1 shows a 2 s undo bar; esc cancels; after 2 s the hook returns allow and the command runs.
54. *OpenCode permission*: Given OC asks to run a command, then the card appears and approving it resumes OC.
55. *Parallel*: Given 2 CC + 2 OC sessions started from the grid, then all 4 show distinct live terminals and the Activity "at once" peak = 4.
56. *Collision*: Given two agents edit the same file within 20 min, then both show a red ring and who_edits lists both.
57. *Auth error*: Given a CC turn ends with "Login expired", then the status is error:auth (never "Finished") and the card offers login.
58. *Schema drift*: Given an unknown OpenCode schema, then a visible "unsupported version" banner shows (not silence).
59. *Host drift*: Given the host CLI lacks a command, then the diagnostics page names it and other features keep working.
60. *Hook reversibility*: Install then uninstall the hook → settings.json is byte-identical to the original.
61. *Recovery*: Kill the UI with 3 running sessions → relaunch → the sessions are still running and attached; quit the daemon → relaunch → offers resume for each.
62. *Quick-send*: With no typeable agent, ⌥⌥ + ↩ shows an explanation.
63. *Mobile*: The phone page requires pairing; an invalid token gets 401; actions honour the undo window.
64. *Performance*: 20 idle agents < 1 % CPU, < 150 MB RAM.
19. Appendix: raw interfaces
19.1 Claude Code hook block written by AgentDock [V]
"hooks": {
 "Notification":      [{"hooks":[{"type":"command","command":"\"$HOME/.agentdock/hook\" claude","timeout":5}]}],
 "PermissionRequest": [{"hooks":[{"type":"command","command":"\"$HOME/.agentdock/hook\" claude","timeout":3600}]}],
 "PostToolUse":       [{"hooks":[{"type":"command","command":"\"$HOME/.agentdock/hook\" claude","timeout":5}]}],
 "SessionEnd":        [{"hooks":[{"type":"command","command":"\"$HOME/.agentdock/hook\" claude","timeout":5}]}],
 "SessionStart":      [{"hooks":[{"type":"command","command":"\"$HOME/.agentdock/hook\" claude","timeout":5}]}],
 "Stop":              [{"hooks":[{"type":"command","command":"\"$HOME/.agentdock/hook\" claude","timeout":5}]}],
 "UserPromptSubmit":  [{"hooks":[{"type":"command","command":"\"$HOME/.agentdock/hook\" claude","timeout":5}]}]
}
	

MCP registration [V] (~/.claude.json → mcpServers.agentdock): {"command": "/Users/<you>/.agentdock/mcp"}.
19.2 MCP tools [V]
Tool
	Input
	Output (verified fields)
	

	

	

	list_agents
	{}
	{count, agents:[{id, agent, kind, name, status: working
	needs_you\
	done\
	idle, folder, place, model?, turn_started_at?, last_activity_at, queued}]}`
	get_agent
	{id}
	above + branch, repository, edited_files[{path, at}], queued_messages[], collision
	

	

	

	who_edits
	{path} (abs, ~ ok)
	{path, agents[{agent_id, name, kind, status, files[], last_edit_at}], collisions[{kind:"same_checkout"…, agent_ids[], agents[], files[]}]}
	

	

	

	recent_turns
	{limit≤50=10, folder?, agent?}
	{turns[{id, agent_id, kind, title, folder, branch?, place, duration_seconds, finished_at, last_words}]}
	

	

	

	send_message
	{agent_id, text, when_done=false}
	error text when untypeable; queued when busy
	

	

	

	start_agent
	{kind: claude
	codex, folder, prompt}`
	{status:"started", kind, folder}; cmux only
	

	

	

Server instructions string (verbatim intent): watch agents; call who_edits before editing shared files; send_message types as if the person had; "Nothing here leaves the Mac." Protocol versions accepted: 2024-11-05 … 2026-07-28.
19.3 iPhone API [V]
* GET /?t=<token> → HTML (401 without a valid token).
* GET /api/state →
{"agents":[{"id":"claude:<uuid>","kind":"claude","title":"…","state":"working|needsInput|done|idle",
           "place":"Claude app","detail":"testbed ⎇ master","turnStartedAt":1795307338169}],
"cards":[{"id":"…","type":"elsewhere|finished|question|permission…","title":"…","text":"…",
          "place":"Claude app","answerable":false,"canReply":false,"options":[],"context":[],"nextSteps":[]}],
"history":[{"id":"…","kind":"claude","seen":false,"length":12.05,"finishedAt":1795308084894,"words":"…"}],
"unseen":3}
	

* POST /api/{undo|seen|choose|step|reply|clear} with JSON {card, number?, text?}.
19.4 Paths
/Applications/AgentDock.app/Contents/MacOS/{AgentDock, agentdock-hook, agentdock-mcp} · ~/.agentdock/{hook, mcp, events.sock, mcp.sock} · ~/Library/Application Support/AgentDock/{activity.json, history.json, trial} · ~/.claude/settings.json.agentdock-backup · URL scheme agentdock:// · App Group 9Q6VR23WUE.com.tihomirvidenov.AgentDock.
19.5 State left on this Mac after testing
All test-time changes were reverted: the Claude Code hook (settings.json byte-identical), MCP registration, OpenCode toggle, iPhone server (port 47811 closed), and cmux socket mode (restored from ~/.config/cmux/cmux.json.bak.*). Re-test changes (hook, cmux automation mode) were reverted the same way. Still present: test cmux workspaces (CLI-permission, CLI-question, OC-readme, OC-tests), the testbed/ folder, and the Claude desktop session "AgentDock test: question + permission" (waiting on its question).
Screenshot gallery (all captures, in order)
Every screenshot taken during testing appears either next to the section that discusses it or below.
  

Figure: Hook and MCP switched on (during test)  (23-claude-hook-and-mcp-enabled.jpg)
  

Figure: "Show OpenCode sessions" on (during test)  (24-opencode-enabled.jpg)