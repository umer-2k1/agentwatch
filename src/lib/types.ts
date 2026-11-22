export type Status = "working" | "needsYou" | "done" | "idle" | "dead" | "unknown";
export type Confidence = "certain" | "inferred" | "unknown";

export interface Evidence { kind: "transcript" | "hook" | "process"; detail: string }
export interface FileEdit { path: string; atMs: number; tool: string }
export interface SubagentInfo { id: string; kind: string; title: string; status: string; elapsedS: number }
export interface CollisionInfo { kind: string; otherId: string; otherTitle: string; file: string | null }

export interface Session {
  id: string; provider: string; title: string; cwd: string; repo: string; branch: string;
  model: string; origin?: string; place?: string;
  turnClosed: boolean; pendingTool: string | null;
  lastActivityMs: number; turnStartedMs: number | null;
  lastWords?: string; firstPrompt?: string; errorKind?: string | null;
  costUsd: number | null; tokensIn: number; tokensOut: number;
  cacheReadTokens?: number; cacheWriteTokens?: number;
  editedFiles?: FileEdit[]; subagents?: SubagentInfo[];
  isSidechain: boolean; readable: boolean;
  status: Status; confidence: Confidence; evidence: Evidence;
  pid: number | null; quietForS: number; turnAgeS: number | null;
  collisions?: CollisionInfo[]; queuedMessages?: string[]; seen?: boolean;
}

export interface CardOption { number: number; label: string; description: string; action: string }

export interface InboxCard {
  id: string; sessionId: string; provider: string;
  cardType: "permission" | "question" | "plan" | "trust" | "auth" | "balance" | "elsewhere" | "finished";
  title: string; repo: string; branch: string; place: string; ageS: number;
  header: string; body: string; command: string | null; question: string | null;
  options: CardOption[]; answerable: boolean; canReply: boolean;
  state: "open" | "pending_undo"; chosenLabel: string | null; undoRemainingMs: number | null;
}

export interface TurnRecord {
  id: string; sessionId: string; provider: string; title: string; folder: string;
  repo: string; branch: string; place: string; model: string;
  startedMs: number; finishedMs: number; durationS: number;
  lastWords: string; outcome: string; errorKind: string | null; seen: boolean;
  tokensIn: number; tokensOut: number; costUsd: number;
}

export interface DayLane { sessionId: string; provider: string; title: string; repo: string; turns: number; hours: number; spans: { startHour: number; durationHours: number }[] }
export interface DaySummary {
  date: string; label: string; weekday: number; hours: number; turns: number; sessions: number;
  maxConcurrent: number; peakTime: string; tokens: number; costUsd: number;
  awayHours: number; waitMinutes: number; waitCount: number;
  hourly: number[]; lanes: DayLane[];
}
export interface ActivityReport {
  totalHours: number; totalTurns: number; totalSessions: number; activeDays: number;
  totalTokens: number; totalCostUsd: number; cacheHitPct: number; currentStreak: number;
  topModel: string; topModelPct: number; days: DaySummary[];
  leverage: { ratio: number; agentHours: number; macHours: number; hereHours: number; awayHours: number };
  waiting: { totalMinutes: number; count: number; longestS: number; medianS: number; under1m: number; m1To5: number; m5To15: number; m15To60: number; over60m: number };
  models: { model: string; provider: string; hours: number; sharePct: number; tokens: number; costUsd: number }[];
  subagents: { total: number; helper: number; explore: number; general: number; topSessionTitle: string; topSessionCount: number };
  thatsLike: { hoursLine: string; tokensLine: string; costLine: string };
  records: {
    longestTurnLabel: string; longestTurnSub: string; longestTurnDate: string;
    mostAtOnce: number; mostAtOnceSub: string; mostAtOnceDate: string;
    busiestDayTurns: number; busiestDaySub: string; busiestDayDate: string;
    bestStreakDays: number; bestStreakSub: string; currentStreakDays: number;
    nightOwlTurns: number; nightOwlSub: string; rhythmLabel: string; rhythmSub: string;
  };
}

export interface AgentCatalogItem {
  id: string; name: string; short: string; tier: "supported" | "preview" | "cmux";
  installed: boolean; path: string; description: string;
  canSee: string; canHistory: string; canAnswer: string; canType: string;
  hookSupported: boolean; mcpSupported: boolean;
}

export interface HostAppItem {
  id: string; name: string; installed: boolean; statusBadge: string; statusOk: boolean;
  summary: string; canSee: boolean; canAnswer: string; canType: boolean; canJump: boolean;
  setupSteps: string[]; diagnostic: string;
}

export interface DiagnosticsReport {
  claudeInstalled: boolean; claudeHookInstalled: boolean; claudeMcpRegistered: boolean;
  opencodeDbExists: boolean; opencodeSchema: string; codexSessionsExist: boolean;
  cmuxInstalled: boolean; cmuxCliProbe: string; tmuxInstalled: boolean;
  hookSocketListening: boolean; iphoneServerRunning: boolean;
}

export interface IphoneStatus { enabled: boolean; port: number; lanIp: string; token: string; url: string; relayUrl: string }
export interface SpawnedView {
  id: string; kind: string; title: string; folder: string; repo: string; branch: string;
  worktree: string | null; pid: number | null; running: boolean; exitCode: number | null;
  lines: string[]; detectedState: string; errorKind: string | null;
  startedMs: number; lastActivityMs: number; resumeCmd: string;
}

export const STATUS_ORDER: Status[] = ["needsYou", "dead", "working", "done", "idle", "unknown"];
export const STATUS_LABEL: Record<Status, string> = {
  needsYou: "needs you", working: "working", done: "done", idle: "idle", dead: "dead", unknown: "unknown",
};

export const VENDOR: Record<string, { label: string; className: string }> = {
  "claude-code": { label: "Claude Code", className: "brand-claude" },
  codex: { label: "Codex", className: "brand-codex" },
  opencode: { label: "OpenCode", className: "brand-opencode" },
  cursor: { label: "Cursor", className: "brand-cursor" },
  "gemini-cli": { label: "Gemini CLI", className: "brand-gemini" },
  "copilot-cli": { label: "Copilot", className: "brand-copilot" },
  shell: { label: "Shell", className: "brand-cursor" },
};

export function vendorOf(id: string) {
  return VENDOR[id] ?? { label: id || "Agent", className: "brand-cursor" };
}

export interface ProviderInfo {
  id: string;
  label: string;
  verification: "verified" | "unverified";
  sessions: number;
}
