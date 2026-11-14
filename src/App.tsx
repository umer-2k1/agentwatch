import { useEffect, useMemo, useState } from "react";
import { Header } from "@/components/Header";
import { GroupSection } from "@/components/GroupSection";
import { SettingsSheet, type Prefs } from "@/components/SettingsSheet";
import { STATUS_ORDER, type ProviderInfo, type Session } from "@/lib/types";
import * as api from "@/lib/bridge";

const DEFAULT_PREFS: Prefs = { theme: "dark", accent: "211 100% 52%", radius: 12, glass: true };

export default function App() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [awaiting, setAwaiting] = useState<Set<string>>(new Set());
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [prefs, setPrefs] = useState<Prefs>(() => {
    try {
      return { ...DEFAULT_PREFS, ...JSON.parse(localStorage.getItem("agentwatch.prefs") || "{}") };
    } catch {
      return DEFAULT_PREFS;
    }
  });

  // ---- appearance: one class + three custom properties, nothing else ----
  useEffect(() => {
    const root = document.documentElement;
    root.classList.toggle("dark", prefs.theme === "dark");
    root.style.setProperty("--accent", prefs.accent);
    root.style.setProperty("--primary", prefs.accent);
    root.style.setProperty("--ring", prefs.accent);
    root.style.setProperty("--radius", `${prefs.radius / 16}rem`);
    localStorage.setItem("agentwatch.prefs", JSON.stringify(prefs));
  }, [prefs]);

  useEffect(() => {
    let stop: undefined | (() => void);
    (async () => {
      const [s, p, a] = await Promise.all([api.getSessions(), api.getProviders(), api.getAwaiting()]);
      setSessions(s);
      setProviders(p);
      setAwaiting(new Set(a));
      stop = await api.onSessions((next) => setSessions(next));
      await api.onFocusAgent(() => {});
    })();
    const t = setInterval(async () => {
      try {
        const [s, a] = await Promise.all([api.getSessions(), api.getAwaiting()]);
        setSessions(s);
        setAwaiting(new Set(a));
      } catch {}
    }, 1500);
    return () => {
      stop?.();
      clearInterval(t);
    };
  }, []);

  const groups = useMemo(() => {
    const sorted = [...sessions].sort(
      (a, b) => STATUS_ORDER.indexOf(a.status) - STATUS_ORDER.indexOf(b.status)
    );
    const map = new Map<string, Session[]>();
    for (const s of sorted) {
      const key = s.repo || "other";
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(s);
    }
    // A group inherits the urgency of its most urgent agent, so "needs you" repos
    // float to the top instead of appearing in filesystem order.
    return [...map.entries()].sort((a, b) => {
      const rank = (items: Session[]) => Math.min(...items.map((i) => STATUS_ORDER.indexOf(i.status)));
      return rank(a[1]) - rank(b[1]);
    });
  }, [sessions]);

  async function handleAnswer(id: string, decision: "allow" | "deny") {
    setAwaiting((prev) => {
      const next = new Set(prev);
      next.delete(id);
      return next;
    });
    await api.answer(id, decision);
  }

  return (
    <div className="app-canvas flex h-full flex-col overflow-hidden">
      <Header sessions={sessions} onSettings={() => setSettingsOpen((o) => !o)} />

      <div className="flex min-h-0 flex-1">
        <main className="flex min-w-0 flex-1 flex-col">
          <div className="min-h-0 flex-1 overflow-y-auto px-3.5 py-3.5">
            {sessions.length === 0 ? (
              <EmptyState />
            ) : (
              groups.map(([repo, items]) => (
                <GroupSection
                  key={repo}
                  repo={repo}
                  sessions={items}
                  awaiting={awaiting}
                  onAnswer={handleAnswer}
                />
              ))
            )}
          </div>

          <footer className="glass flex shrink-0 items-center justify-between border-t px-3.5 py-1.5 text-[10.5px] text-muted-foreground">
            <div className="flex gap-3">
              {providers.map((p) => (
                <span key={p.id} className="flex items-center gap-1.5">
                  <span
                    className={
                      p.verification === "verified"
                        ? "rounded-full border border-status-done/40 px-1.5 text-status-done"
                        : "rounded-full border border-status-needsYou/40 px-1.5 text-status-needsYou"
                    }
                  >
                    {p.verification}
                  </span>
                  {p.label}
                </span>
              ))}
            </div>
            <span>Click a card for why</span>
          </footer>
        </main>

        <SettingsSheet
          prefs={prefs}
          onChange={setPrefs}
          open={settingsOpen}
        />
      </div>
    </div>
  );
}

function EmptyState() {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-1.5 text-center text-muted-foreground">
      <p className="text-[13.5px] font-medium text-foreground/80">No agents active in the last 48h</p>
      <p className="text-[12px]">
        Start Claude Code, Codex or OpenCode and it appears here.
      </p>
    </div>
  );
}
