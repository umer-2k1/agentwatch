import type { ProviderInfo, Session } from "./types";

/** Fixture used only by the browser dev server (`pnpm dev`), never inside Tauri. */
export function mockSessions(): Session[] {
  const now = Date.now();
  const base = {
    cwd: "/Users/me/code/shop-web",
    turnClosed: true,
    pendingTool: null,
    turnStartedMs: null,
    costUsd: null,
    tokensIn: 0,
    tokensOut: 0,
    isSidechain: false,
    readable: true,
    lastActivityMs: now,
  };
  const mk = (o: Partial<Session> & { id: string; title: string }): Session =>
    ({ ...base, ...o, quietForS: o.quietForS ?? 0 }) as Session;

  return [
    mk({ id: "mock-1", provider: "claude-code", title: "pricing page layout", repo: "shop-web",
         branch: "feature/pricing", model: "claude-opus-5", status: "needsYou", confidence: "certain",
         pid: 48213, quietForS: 74, evidence: { kind: "hook", detail: "Claude needs your permission to use Bash" } }),
    mk({ id: "mock-2", provider: "claude-code", title: "fix flaky login test", repo: "shop-web",
         branch: "main", model: "claude-opus-5", status: "working", confidence: "certain",
         pid: 48301, quietForS: 3, turnAgeS: 1327, evidence: { kind: "process", detail: "pid 48301 alive" } }),
    mk({ id: "mock-3", provider: "codex", title: "research pricing pages", repo: "shop-api",
         branch: "main", model: "gpt-5-codex", status: "done", confidence: "inferred",
         pid: 48355, quietForS: 96, evidence: { kind: "transcript", detail: "turn ended (task_complete)" } }),
    mk({ id: "mock-4", provider: "codex", title: "add stripe checkout", repo: "shop-api",
         branch: "main", model: "gpt-5-codex", status: "working", confidence: "certain",
         pid: 48360, quietForS: 12, evidence: { kind: "process", detail: "pid 48360 alive" } }),
    mk({ id: "mock-5", provider: "opencode", title: "migrate to SwiftData", repo: "notes-ios",
         branch: "swiftdata", model: "claude-sonnet-5", status: "dead", confidence: "certain",
         pid: 48402, quietForS: 640, evidence: { kind: "process", detail: "pid 48402 is gone" } }),
    mk({ id: "mock-6", provider: "cursor", title: "tidy the README", repo: "notes-ios",
         branch: "readme", model: "claude-haiku-5", status: "unknown", confidence: "unknown",
         pid: null, quietForS: 14400, evidence: { kind: "process", detail: "turn never closed, no process" } }),
    mk({ id: "mock-7", provider: "gemini-cli", title: "update release notes", repo: "site",
         branch: "release/2.4", model: "gemini-3-pro", status: "idle", confidence: "inferred",
         pid: 48500, quietForS: 2400, evidence: { kind: "transcript", detail: "quiet for 40m" } }),
  ];
}

export function mockProviders(): ProviderInfo[] {
  return [
    { id: "claude-code", label: "Claude Code", verification: "verified", sessions: 2 },
    { id: "codex", label: "Codex", verification: "verified", sessions: 2 },
    { id: "opencode", label: "OpenCode", verification: "verified", sessions: 1 },
  ];
}
