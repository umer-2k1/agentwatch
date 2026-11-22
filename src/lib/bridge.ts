import type {
  ActivityReport, AgentCatalogItem, DiagnosticsReport, HostAppItem, InboxCard, IphoneStatus,
  AgentCatalogItem as _A, ProviderInfo, Session, SpawnedView, TurnRecord,
} from "./types";
import { mockActivity, mockAgents, mockDiagnostics, mockHosts, mockInbox, mockProviders, mockSessions, mockTurns } from "./mock";

declare global {
  interface Window {
    __TAURI__?: {
      core: { invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T> };
      event: { listen: <T>(e: string, cb: (ev: { payload: T }) => void) => Promise<() => void> };
    };
  }
}

export const inTauri = () => typeof window !== "undefined" && !!window.__TAURI__;
const call = <T,>(cmd: string, args?: Record<string, unknown>, fallback?: T): Promise<T> =>
  inTauri() ? window.__TAURI__!.core.invoke<T>(cmd, args) : Promise.resolve(fallback as T);

export const getSessions = () => call<Session[]>("get_sessions", undefined, mockSessions());
export const getProviders = () => call<ProviderInfo[]>("get_providers", undefined, mockProviders());
export const getInboxCards = () => call<InboxCard[]>("get_inbox_cards", undefined, mockInbox());
export const getHistoryTurns = () => call<TurnRecord[]>("get_history_turns", undefined, mockTurns());
export const getActivity = () => call<ActivityReport | null>("get_activity_report", undefined, mockActivity());
export const getAgents = () => call<AgentCatalogItem[]>("get_agent_catalog", undefined, mockAgents());
export const getHosts = () => call<HostAppItem[]>("get_host_catalog", undefined, mockHosts());
export const getDiagnostics = () => call<DiagnosticsReport>("get_diagnostics", undefined, mockDiagnostics());
export const getSpawned = () => call<SpawnedView[]>("get_spawned_sessions", undefined, []);

export const answer = (sessionId: string, decision: string, label?: string) =>
  call<boolean>("answer", { sessionId, decision, label }, true);
export const undoAnswer = (sessionId: string) => call<boolean>("undo_answer", { sessionId }, true);
export const markSeen = (sessionId: string) => call<void>("mark_seen", { sessionId }, undefined);
export const markAllSeen = () => call<void>("mark_all_seen", undefined, undefined);
export const sendMessage = (sessionId: string, text: string, whenDone = false) =>
  call<string>("send_message", { sessionId, text, whenDone }, "queued");
export const spawnSession = (kind: string, folder: string, prompt: string, worktree = false) =>
  call<SpawnedView | null>("spawn_session", { kind, folder, prompt, createWorktree: worktree }, null);
export const stopSpawned = (sessionId: string) => call<boolean>("stop_spawned_session", { sessionId }, true);
export const sendPtyInput = (sessionId: string, text: string) =>
  call<void>("send_pty_input", { sessionId, text }, undefined);
export const simulatePrompt = (kind: "permission" | "question") =>
  call<string>("simulate_prompt", { kind }, "");
export const jumpToAgent = (place: string, cwd: string) =>
  call<string>("jump_to_agent", { place, cwd }, "activated");
export const hooksStatus = () => call<{ installed: boolean } | null>("hooks_status", undefined, null);
export const hooksInstall = () => call<string>("hooks_install", undefined, "installed");
export const hooksUninstall = () => call<boolean>("hooks_uninstall", undefined, true);
export const mcpStatus = () => call<boolean>("mcp_status", undefined, false);
export const mcpToggle = (enable: boolean) => call<boolean>("mcp_toggle", { enable }, enable);
export const iphoneStatus = () => call<IphoneStatus | null>("iphone_status", undefined, null);
export const iphoneToggle = (enabled: boolean) => call<IphoneStatus | null>("iphone_toggle", { enabled }, null);
export const iphoneRotate = () => call<IphoneStatus | null>("iphone_rotate_token", undefined, null);
export const setStripPosition = (position: string) => call<void>("set_strip_position", { position }, undefined);

export async function onSessions(cb: (s: Session[]) => void): Promise<() => void> {
  if (!inTauri()) return () => {};
  return window.__TAURI__!.event.listen<Session[]>("sessions", (e) => cb(e.payload));
}
export async function onNavigate(cb: (v: string) => void): Promise<() => void> {
  if (!inTauri()) return () => {};
  return window.__TAURI__!.event.listen<string>("navigate", (e) => cb(e.payload));
}
export async function onFocusAgent(cb: (id: string) => void): Promise<() => void> {
  if (!inTauri()) return () => {};
  return window.__TAURI__!.event.listen<string>("focus-agent", (e) => cb(e.payload));
}
