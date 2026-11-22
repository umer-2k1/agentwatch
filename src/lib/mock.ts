import type {
  ActivityReport, AgentCatalogItem, DiagnosticsReport, HostAppItem, InboxCard, ProviderInfo,
  Session, TurnRecord,
} from "./types";

const now = Date.now();

const base = {
  cwd: "/Users/me/code/shop-web", turnClosed: true, pendingTool: null, turnStartedMs: null,
  costUsd: null, tokensIn: 0, tokensOut: 0, isSidechain: false, readable: true,
  lastActivityMs: now, branch: "main", model: "claude-opus-5",
  origin: "discovered", place: "Terminal", lastWords: "", collisions: [], queuedMessages: [], seen: false,
};

export function mockSessions(): Session[] {
  const mk = (o: Partial<Session> & { id: string; title: string }): Session =>
    ({ ...base, quietForS: 0, ...o }) as Session;

  return [
    mk({
      id: "claude-code:a1", provider: "claude-code", title: "Pricing page layout", repo: "shop-web",
      branch: "feature/pricing", status: "needsYou", confidence: "certain", pid: 48213, quietForS: 74,
      evidence: { kind: "hook", detail: "Bash: pnpm test -- --watch=false" },
    }),
    mk({
      id: "claude-code:a2", provider: "claude-code", title: "Add Stripe checkout", repo: "shop-api",
      branch: "main", model: "claude-opus-5", status: "working", confidence: "certain", pid: 48301,
      quietForS: 3, turnAgeS: 1327, tokensIn: 320_000, tokensOut: 12_400, cacheReadTokens: 280_000,
      subagents: [
        { id: "s1", kind: "Explore", title: "Find where the cart total is computed", status: "running", elapsedS: 48 },
        { id: "s2", kind: "Plan", title: "Draft the Stripe webhook handler", status: "running", elapsedS: 112 },
      ],
      editedFiles: [{ path: "/Users/me/code/shop-api/src/SessionStore.swift", atMs: now - 600_000, tool: "Edit" }],
      evidence: { kind: "process", detail: "pid 48301 alive" },
    }),
    mk({
      id: "codex:b1", provider: "codex", title: "Fix the flaky login test", repo: "shop-api",
      branch: "main", model: "gpt-5-codex", status: "working", confidence: "certain", pid: 48360,
      quietForS: 2, place: "Terminal",
      editedFiles: [{ path: "/Users/me/code/shop-api/src/SessionStore.swift", atMs: now - 500_000, tool: "Write" }],
      evidence: { kind: "process", detail: "pid 48360 alive" },
    }),
    mk({
      id: "opencode:c1", provider: "opencode", title: "Research pricing pages", repo: "shop-web",
      model: "gemini-4-argon", status: "done", confidence: "inferred", pid: 48355, quietForS: 96,
      tokensIn: 129_671, tokensOut: 20_233, place: "OpenCode",
      lastWords: "Compared 14 pricing pages. Most use three tiers with the middle one highlighted; four show yearly prices first. Summary table in `pricing-research.md`.",
      evidence: { kind: "transcript", detail: "turn ended (task_complete)" },
    }),
    mk({
      id: "opencode:c2", provider: "opencode", title: "Tidy the README", repo: "notes-ios",
      branch: "readme", model: "claude-sonnet-5", status: "idle", confidence: "inferred",
      pid: 48402, quietForS: 2400, place: "OpenCode", seen: true,
      evidence: { kind: "transcript", detail: "quiet for 40m" },
    }),
    mk({
      id: "claude-code:a3", provider: "claude-code", title: "Migrate settings to SwiftData", repo: "notes-ios",
      branch: "swiftdata", status: "dead", confidence: "certain", pid: 48420, quietForS: 640,
      errorKind: "auth", lastWords: "Login expired · Please run /login",
      evidence: { kind: "transcript", detail: "error:auth — Login expired" },
    }),
  ];
}

export function mockProviders(): ProviderInfo[] {
  return [
    { id: "claude-code", label: "Claude Code", verification: "verified", sessions: 3 },
    { id: "codex", label: "Codex", verification: "verified", sessions: 1 },
    { id: "opencode", label: "OpenCode", verification: "verified", sessions: 2 },
  ];
}

export function mockInbox(): InboxCard[] {
  return [
    {
      id: "claude-code:a1", sessionId: "claude-code:a1", provider: "claude-code", cardType: "permission",
      title: "Run this exact Bash command", repo: "shop-web", branch: "main", place: "Terminal", ageS: 74,
      header: "PRINT A TEST STRING", body: "pnpm test -- --watch=false", command: "pnpm test -- --watch=false",
      question: "Do you want to proceed?",
      options: [
        { number: 1, label: "Yes", description: "Allow this command", action: "allow" },
        { number: 2, label: "No", description: "Decline in AgentWatch", action: "deny" },
      ],
      answerable: true, canReply: true, state: "open", chosenLabel: null, undoRemainingMs: null,
    },
    {
      id: "claude-code:a4", sessionId: "claude-code:a4", provider: "claude-code", cardType: "question",
      title: "AskUserQuestion", repo: "shop-web", branch: "main", place: "Terminal", ageS: 30,
      header: "LANGUAGE", body: "Which greeting language?", command: null,
      question: "Which greeting language?",
      options: [
        { number: 1, label: "English", description: "en", action: "choose" },
        { number: 2, label: "Spanish", description: "es", action: "choose" },
        { number: 3, label: "French", description: "fr", action: "choose" },
      ],
      answerable: true, canReply: true, state: "open", chosenLabel: null, undoRemainingMs: null,
    },
    {
      id: "opencode:c1", sessionId: "opencode:c1", provider: "opencode", cardType: "finished",
      title: "Research pricing pages", repo: "shop-web", branch: "main", place: "OpenCode", ageS: 96,
      header: "FINISHED", body: "Compared 14 pricing pages. Most use three tiers with the middle one highlighted; four show yearly prices first.",
      command: null, question: null, options: [], answerable: false, canReply: true, state: "open",
      chosenLabel: null, undoRemainingMs: null,
    },
    {
      id: "claude-code:a3", sessionId: "claude-code:a3", provider: "claude-code", cardType: "auth",
      title: "Migrate settings to SwiftData", repo: "notes-ios", branch: "swiftdata", place: "Terminal", ageS: 640,
      header: "ERROR · AUTH", body: "Login expired · Please run /login", command: null, question: null,
      options: [], answerable: false, canReply: false, state: "open", chosenLabel: null, undoRemainingMs: null,
    },
  ];
}

export function mockTurns(): TurnRecord[] {
  const day = 86_400_000;
  return [
    {
      id: "t1", sessionId: "opencode:c1", provider: "opencode", title: "Research pricing pages",
      folder: "/Users/me/code/shop-web", repo: "shop-web", branch: "main", place: "OpenCode",
      model: "gemini-4-argon", startedMs: now - 4 * 60_000, finishedMs: now - 96_000, durationS: 144,
      lastWords: "Compared 14 pricing pages. Most use three tiers.", outcome: "ok", errorKind: null,
      seen: false, tokensIn: 129_671, tokensOut: 20_233, costUsd: 0.38,
    },
    {
      id: "t2", sessionId: "claude-code:a3", provider: "claude-code", title: "Migrate settings to SwiftData",
      folder: "/Users/me/code/notes-ios", repo: "notes-ios", branch: "swiftdata", place: "Terminal",
      model: "claude-opus-5", startedMs: now - day - 40 * 60_000, finishedMs: now - day, durationS: 1700,
      lastWords: "Login expired · Please run /login", outcome: "error", errorKind: "auth",
      seen: false, tokensIn: 44_000, tokensOut: 3_100, costUsd: 0.21,
    },
  ];
}

export function mockActivity(): ActivityReport {
  const day = 86_400_000;
  const days = Array.from({ length: 365 }, (_, i) => {
    const t = now - (364 - i) * day;
    const d = new Date(t);
    const seeded = (i * 37) % 11;
    const hours = seeded > 7 ? 0 : +(seeded * 1.7 + (i % 5) * 0.4).toFixed(1);
    return {
      date: d.toISOString().slice(0, 10),
      label: d.toLocaleDateString(undefined, { weekday: "long", day: "numeric", month: "long" }),
      weekday: (d.getDay() + 6) % 7,
      hours,
      turns: Math.round(hours * 6),
      sessions: Math.max(1, Math.round(hours / 2)),
      maxConcurrent: Math.min(6, 1 + (i % 5)),
      peakTime: "11:45",
      tokens: Math.round(hours * 1_200_000),
      costUsd: +(hours * 1.4).toFixed(2),
      awayHours: +(hours * 0.3).toFixed(1),
      waitMinutes: +(hours * 2).toFixed(0),
      waitCount: Math.round(hours * 2),
      hourly: Array.from({ length: 24 }, (_, h) => {
        const peak = Math.exp(-Math.pow(h - 13, 2) / 18);
        return +(hours * peak).toFixed(2);
      }),
      lanes: seeded > 7 ? [] : [
        { sessionId: "l1", provider: "claude-code", title: "Fix the bug in app.py, then run the tests", repo: "shop-api", turns: 4, hours: 2.1, spans: [{ startHour: 9.5, durationHours: 2.1 }] },
        { sessionId: "l2", provider: "opencode", title: "Before doing anything, use the plan tool", repo: "shop-web", turns: 3, hours: 1.4, spans: [{ startHour: 11, durationHours: 1.4 }] },
        { sessionId: "l3", provider: "codex", title: "btw: can I use my system right now", repo: "notes-ios", turns: 2, hours: 0.9, spans: [{ startHour: 14.2, durationHours: 0.9 }] },
      ],
    };
  });

  return {
    totalHours: 81, totalTurns: 1621, totalSessions: 184, activeDays: 43,
    totalTokens: 191_000_000, totalCostUsd: 114.2, cacheHitPct: 97, currentStreak: 4,
    topModel: "claude-opus-5", topModelPct: 42, days,
    leverage: { ratio: 0.9, agentHours: 81, macHours: 90, hereHours: 58, awayHours: 23 },
    waiting: { totalMinutes: 36, count: 31, longestS: 2520, medianS: 180, under1m: 4, m1To5: 21, m5To15: 4, m15To60: 2, over60m: 0 },
    models: [
      { model: "claude-opus-5", provider: "claude-code", hours: 34, sharePct: 42, tokens: 80_000_000, costUsd: 52 },
      { model: "gpt-5-codex", provider: "codex", hours: 20, sharePct: 25, tokens: 44_000_000, costUsd: 30 },
      { model: "gemini-4-argon", provider: "opencode", hours: 14, sharePct: 17, tokens: 36_000_000, costUsd: 18 },
      { model: "claude-sonnet-5", provider: "claude-code", hours: 13, sharePct: 16, tokens: 31_000_000, costUsd: 14 },
    ],
    subagents: { total: 2203, helper: 900, explore: 640, general: 663, topSessionTitle: "Release notes for 2.4", topSessionCount: 4 },
    thatsLike: {
      hoursLine: "81 h of agent work: 10 full working days",
      tokensLine: "191M tokens read and written: War and Peace 245 times over",
      costLine: "$114.20 at API prices: what the tokens would have cost by the request",
    },
    records: {
      longestTurnLabel: "8h 22m", longestTurnSub: "Claude Code · Add Stripe checkout", longestTurnDate: "2026-01-26",
      mostAtOnce: 6, mostAtOnceSub: "6 at once", mostAtOnceDate: "2026-09-30",
      busiestDayTurns: 202, busiestDaySub: "202 turns · 29h of agent work", busiestDayDate: "2026-10-13",
      bestStreakDays: 21, bestStreakSub: "current streak is 4 days", currentStreakDays: 4,
      nightOwlTurns: 902, nightOwlSub: "latest at 4:55am", rhythmLabel: "Mondays, 11:00",
      rhythmSub: "Mondays have 26% of your turns, peaking at 11:00",
    },
  };
}

export function mockAgents(): AgentCatalogItem[] {
  const mk = (id: string, name: string, short: string, tier: "supported" | "preview" | "cmux", installed: boolean, path: string, desc: string, canAnswer: string, canType: string): AgentCatalogItem => ({
    id, name, short, tier, installed, path, description: desc,
    canSee: "yes", canHistory: "yes", canAnswer, canType,
    hookSupported: id === "claude-code" || id === "codex", mcpSupported: id === "claude-code",
  });
  return [
    mk("claude-code", "Claude Code", "✳", "supported", true, "~/.local/bin/claude", "Reads Claude Code's own session records in any terminal and in the Claude app. The hook adds questions and permission prompts.", "yes", "yes"),
    mk("codex", "Codex", "⬡", "supported", true, "~/.codex", "Reads Codex rollout files in the Codex app and CLI.", "yes", "yes"),
    mk("opencode", "OpenCode", "{ }", "supported", true, "~/.opencode/bin/opencode", "First-class v2 schema support with session_v2 + session_message. Answerable inside AgentWatch PTY sessions.", "yes (PTY)", "yes (PTY)"),
    mk("cursor", "Cursor", "▷", "preview", true, "/Applications/Cursor.app", "Cursor's agent runs Claude Code's hooks.", "not yet", "not yet"),
    mk("gemini-cli", "Gemini CLI", "✦", "preview", false, "Not installed", "Reads Gemini CLI's own project session files.", "not yet", "not yet"),
    mk("copilot-cli", "GitHub Copilot", "⧉", "preview", false, "Not installed", "Reads Copilot CLI session files.", "not yet", "not yet"),
    mk("kilo-code", "Kilo Code", "K", "preview", false, "Not installed", "Reads kilo.db session store.", "not yet", "not yet"),
    mk("antigravity", "Antigravity", "▲", "preview", false, "Not installed", "Reads agy conversation files.", "not yet", "not yet"),
    mk("goose", "Goose", "G", "preview", false, "Not installed", "Reads Block Goose SQLite store.", "not yet", "not yet"),
    mk("cline", "Cline", "C", "preview", false, "Not installed", "Reads Cline VS Code tasks.", "not yet", "not yet"),
  ];
}

export function mockHosts(): HostAppItem[] {
  const mk = (id: string, name: string, installed: boolean, badge: string, summary: string, canType: boolean, diag: string): HostAppItem => ({
    id, name, installed, statusBadge: badge, statusOk: installed, summary, canSee: true,
    canAnswer: canType ? "Full" : "With Claude hook", canType, canJump: true,
    setupSteps: canType ? ["Run your agents inside this host.", "AgentWatch reads and types automatically."] : ["Turn on the Claude Code hook in Settings › Agents.", "Run tmux inside this host for full typing."],
    diagnostic: diag,
  });
  return [
    mk("cmux", "cmux", true, "Ready (workspace CLI)", "Full support with capability-probed cmux workspace subcommands.", true, "cmux 2.1 · modern workspace subcommands"),
    mk("tmux", "tmux", true, "Installed", "Types into panes directly through tmux send-keys.", true, "/opt/homebrew/bin/tmux"),
    mk("iterm", "iTerm", true, "Installed", "AppleScript locates the exact tab by tty.", true, "/Applications/iTerm.app"),
    mk("wezterm", "WezTerm", false, "Not installed", "wezterm cli send-text.", false, "not found"),
    mk("terminal", "Terminal", true, "Built-in · Hook", "Shows agents and answers Claude via hook.", false, "/System/Applications/Utilities/Terminal.app"),
    mk("ghostty", "Ghostty", false, "Not installed", "Shows agents and answers via hook.", false, "not found"),
    mk("claude-app", "Claude app", true, "Installed", "Claude desktop sessions show up automatically.", false, "/Applications/Claude.app"),
  ];
}

export function mockDiagnostics(): DiagnosticsReport {
  return {
    claudeInstalled: true, claudeHookInstalled: false, claudeMcpRegistered: false,
    opencodeDbExists: true, opencodeSchema: "OpenCode v2 schema (session_v2 + session_message) — verified & active",
    codexSessionsExist: true, cmuxInstalled: true, cmuxCliProbe: "cmux installed · modern `workspace` subcommands supported",
    tmuxInstalled: true, hookSocketListening: true, iphoneServerRunning: false,
  };
}
