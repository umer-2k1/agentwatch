import { useEffect, useMemo, useState } from "react";
import { Activity, Inbox, LayoutGrid, Settings } from "lucide-react";
import { InboxView } from "@/components/InboxView";
import { ActivityView } from "@/components/ActivityView";
import { SettingsView, DEFAULT_PREFS, type Prefs } from "@/components/SettingsView";
import { SessionsView } from "@/components/SessionsView";
import { cn } from "@/lib/utils";
import type {
  ActivityReport, AgentCatalogItem, DiagnosticsReport, HostAppItem, InboxCard, IphoneStatus,
  Session, SpawnedView, TurnRecord,
} from "@/lib/types";
import * as api from "@/lib/bridge";

type View = "inbox" | "sessions" | "activity" | "settings";

export default function App() {
  const [view, setView] = useState<View>("inbox");
  const [sessions, setSessions] = useState<Session[]>([]);
  const [cards, setCards] = useState<InboxCard[]>([]);
  const [turns, setTurns] = useState<TurnRecord[]>([]);
  const [activity, setActivity] = useState<ActivityReport | null>(null);
  const [agents, setAgents] = useState<AgentCatalogItem[]>([]);
  const [hosts, setHosts] = useState<HostAppItem[]>([]);
  const [diagnostics, setDiagnostics] = useState<DiagnosticsReport | null>(null);
  const [iphone, setIphone] = useState<IphoneStatus | null>(null);
  const [spawned, setSpawned] = useState<SpawnedView[]>([]);

  const [prefs, setPrefs] = useState<Prefs>(() => {
    try { return { ...DEFAULT_PREFS, ...JSON.parse(localStorage.getItem("agentwatch.prefs") || "{}") }; }
    catch { return DEFAULT_PREFS; }
  });

  useEffect(() => {
    const root = document.documentElement;
    const dark = prefs.theme === "system"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
      : prefs.theme === "dark";
    root.classList.toggle("dark", dark);
    root.style.setProperty("--primary", prefs.accent);
    root.style.setProperty("--ring", prefs.accent);
    root.style.setProperty("--radius", `${prefs.radius / 16}rem`);
    localStorage.setItem("agentwatch.prefs", JSON.stringify(prefs));
  }, [prefs]);

  useEffect(() => {
    let stop: undefined | (() => void);
    (async () => {
      const [s, c, t, a, ag, ho, dg, ip, sp] = await Promise.all([
        api.getSessions(), api.getInboxCards(), api.getHistoryTurns(), api.getActivity(),
        api.getAgents(), api.getHosts(), api.getDiagnostics(), api.iphoneStatus(), api.getSpawned(),
      ]);
      setSessions(s); setCards(c); setTurns(t); setActivity(a);
      setAgents(ag); setHosts(ho); setDiagnostics(dg); setIphone(ip); setSpawned(sp);
      stop = await api.onSessions(setSessions);
      await api.onNavigate((v) => {
        if (v === "ask" || v === "inbox") setView("inbox");
        else if (v === "activity") setView("activity");
        else if (v === "settings" || v === "tour" || v === "diagnostics" || v === "roadmap") setView("settings");
        else if (v === "sessions") setView("sessions");
      });
    })();

    const poll = setInterval(async () => {
      try {
        const [c, t, a, sp] = await Promise.all([
          api.getInboxCards(), api.getHistoryTurns(), api.getActivity(), api.getSpawned(),
        ]);
        setCards(c); setTurns(t); setActivity(a); setSpawned(sp);
      } catch {}
    }, 1200);
    return () => { stop?.(); clearInterval(poll); };
  }, []);

  const badge = useMemo(() => cards.filter((c) => c.answerable).length, [cards]);

  async function handleAnswer(id: string, decision: string, label?: string) {
    await api.answer(id, decision, label);
    setCards((prev) =>
      prev.map((c) => (c.id === id ? { ...c, state: "pending_undo", chosenLabel: label ?? null, undoRemainingMs: 2000 } : c))
    );
  }

  return (
    <div className="app-canvas flex h-full flex-col overflow-hidden">
      {/* 36 px chrome bar — no sidebar, exactly like AgentDock's panel */}
      <header className="glass flex h-9 shrink-0 items-center gap-1 border-b border-border/70 px-2.5">
        <span className="mr-1 h-1.5 w-1.5 rounded-full bg-primary" />
        <span className="text-[12px] font-semibold tracking-tight">AgentWatch</span>

        <div className="ml-auto flex items-center gap-0.5">
          <Tab active={view === "inbox"} onClick={() => setView("inbox")} icon={Inbox} label="Inbox" badge={badge} />
          <Tab active={view === "sessions"} onClick={() => setView("sessions")} icon={LayoutGrid} label="Sessions" />
          <Tab active={view === "activity"} onClick={() => setView("activity")} icon={Activity} label="Activity" />
          <Tab active={view === "settings"} onClick={() => setView("settings")} icon={Settings} label="Settings" />
        </div>
      </header>

      <main className="flex min-h-0 flex-1 flex-col">
        {view === "inbox" && (
          <InboxView
            cards={cards}
            turns={turns}
            onAnswer={handleAnswer}
            onMarkSeen={(id) => { api.markSeen(id); setCards((p) => p.filter((c) => c.id !== id)); }}
            onMarkAllSeen={() => { api.markAllSeen(); setCards([]); }}
            onJump={(place, cwd) => api.jumpToAgent(place, cwd)}
          />
        )}
        {view === "sessions" && (
          <SessionsView
            sessions={sessions}
            spawned={spawned}
            onSpawn={async (k, f, p, w) => { await api.spawnSession(k, f, p, w); }}
            onStop={(id) => api.stopSpawned(id)}
            onSend={(id, text) => api.sendPtyInput(id, text)}
          />
        )}
        {view === "activity" && <ActivityView report={activity} />}
        {view === "settings" && (
          <SettingsView prefs={prefs} setPrefs={setPrefs} agents={agents} hosts={hosts} diagnostics={diagnostics} iphone={iphone} />
        )}
      </main>
    </div>
  );
}

function Tab({
  active, onClick, icon: Icon, label, badge,
}: {
  active: boolean; onClick: () => void; icon: typeof Inbox; label: string; badge?: number;
}) {
  return (
    <button
      onClick={onClick}
      title={label}
      className={cn(
        "flex items-center gap-1.5 rounded-md px-2 py-1 text-[11px] transition-colors",
        active ? "bg-secondary/90 text-foreground" : "text-muted-foreground hover:bg-secondary/40 hover:text-foreground"
      )}
    >
      <Icon className="h-3 w-3" />
      <span className="hidden sm:inline">{label}</span>
      {badge ? (
        <span className="rounded-full bg-status-needsYou px-1 text-[9px] font-bold leading-4 text-black">{badge}</span>
      ) : null}
    </button>
  );
}
