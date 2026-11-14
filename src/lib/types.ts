export type Status = "working" | "needsYou" | "done" | "idle" | "dead" | "unknown";
export type Confidence = "certain" | "inferred" | "unknown";

export interface Evidence {
  kind: "transcript" | "hook" | "process";
  detail: string;
}

export interface SessionFact {
  id: string;
  provider: string;
  title: string;
  cwd: string;
  repo: string;
  branch: string;
  model: string;
  turnClosed: boolean;
  pendingTool: string | null;
  lastActivityMs: number;
  turnStartedMs: number | null;
  costUsd: number | null;
  tokensIn: number;
  tokensOut: number;
  isSidechain: boolean;
  readable: boolean;
}

export interface Session extends SessionFact {
  status: Status;
  confidence: Confidence;
  evidence: Evidence;
  pid: number | null;
  quietForS: number;
  turnAgeS: number | null;
}

export interface ProviderInfo {
  id: string;
  label: string;
  verification: "verified" | "unverified";
  sessions: number;
}

export const STATUS_ORDER: Status[] = ["needsYou", "dead", "working", "done", "idle", "unknown"];

export const STATUS_LABEL: Record<Status, string> = {
  needsYou: "needs you",
  working: "working",
  done: "done",
  idle: "idle",
  dead: "dead",
  unknown: "unknown",
};

export const VENDOR: Record<string, { label: string; className: string }> = {
  "claude-code": { label: "Claude Code", className: "brand-claude" },
  codex: { label: "Codex", className: "brand-codex" },
  opencode: { label: "OpenCode", className: "brand-opencode" },
  cursor: { label: "Cursor", className: "brand-cursor" },
  "gemini-cli": { label: "Gemini CLI", className: "brand-gemini" },
  "copilot-cli": { label: "Copilot", className: "brand-copilot" },
};

export function vendorOf(id: string) {
  return VENDOR[id] ?? { label: id || "Agent", className: "brand-cursor" };
}
