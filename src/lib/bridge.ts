import type { ProviderInfo, Session } from "./types";
import { mockProviders, mockSessions } from "./mock";

declare global {
  interface Window {
    __TAURI__?: {
      core: { invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T> };
      event: { listen: <T>(e: string, cb: (ev: { payload: T }) => void) => Promise<() => void> };
    };
  }
}

/** True when running inside the Tauri shell. */
export const inTauri = () => typeof window !== "undefined" && !!window.__TAURI__;

/**
 * Browser fallback so `pnpm dev` renders a real board for design work and screenshots.
 * Inside Tauri this is never used.
 */
const mock = () => ({
  sessions: mockSessions(),
  providers: mockProviders(),
  awaiting: ["mock-1"],
});

export async function getSessions(): Promise<Session[]> {
  if (!inTauri()) return mock().sessions;
  return window.__TAURI__!.core.invoke<Session[]>("get_sessions");
}

export async function getProviders(): Promise<ProviderInfo[]> {
  if (!inTauri()) return mock().providers;
  return window.__TAURI__!.core.invoke<ProviderInfo[]>("get_providers");
}

export async function getAwaiting(): Promise<string[]> {
  if (!inTauri()) return mock().awaiting;
  return window.__TAURI__!.core.invoke<string[]>("awaiting");
}

export async function answer(sessionId: string, decision: "allow" | "deny"): Promise<boolean> {
  if (!inTauri()) return true;
  return window.__TAURI__!.core.invoke<boolean>("answer", { sessionId, decision });
}

export async function undoAnswer(sessionId: string): Promise<boolean> {
  if (!inTauri()) return true;
  return window.__TAURI__!.core.invoke<boolean>("undo_answer", { sessionId });
}

export async function onSessions(cb: (s: Session[]) => void): Promise<() => void> {
  if (!inTauri()) return () => {};
  return window.__TAURI__!.event.listen<Session[]>("sessions", (e) => cb(e.payload));
}

export async function onFocusAgent(cb: (id: string) => void): Promise<() => void> {
  if (!inTauri()) return () => {};
  return window.__TAURI__!.event.listen<string>("focus-agent", (e) => cb(e.payload));
}
