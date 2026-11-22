import { useEffect, useMemo, useState } from "react";
import { Sidebar, type View } from "@/components/Sidebar";
import { InboxView } from "@/components/InboxView";
import { ActivityView } from "@/components/ActivityView";
import { SettingsView, DEFAULT_PREFS, type Prefs } from "@/components/SettingsView";
import { SessionsView } from "@/components/SessionsView";
import type {
  ActivityReport, AgentCatalogItem, DiagnosticsReport, HostAppItem, InboxCard, IphoneStatus,
  Session, SpawnedView, TurnRecord,
} from "@/lib/types";
import * as api from "@/lib/bridge";

export default function App() {
  const [view, setView] = useState<View>("sessions");
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
    try {
      return { ...DEFAULT_PREFS, ...JSON.parse(localStorage.getItem("agentwatch.prefs") || "{}") };
    } catch {
      return DEFAULT_PREFS;
    }
  });

  // appearance
  useEffect(() => {
    const root = document.documentElement;
    const dark =
      prefs.theme === "system"
        ? window.matchMedia("(prefers-color-scheme: dark)").matches
        : prefs.theme === "dark";
    root.classList.toggle("dark", dark);
    root.style.setProperty("--primary", prefs.accent);
    root.style.setProperty("--ring", prefs.accent);
    root.style.setProperty("--radius", `${prefs.radius / 16}rem`);
    localStorage.setItem("agentwatch.prefs", JSON.stringify(prefs));
  }, [prefs]);

  // data
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
        if (v === "ask" || v === "inbox" || v === "activity" || v === "settings" || v === "sessions") {
          setView(v === "ask" ? "inbox" : (v as View));
        } else if (v === "tour" || v === "diagnostics" || v === "roadmap") {
          setView("settings");
        }
      });
    })();

    const poll = setInterval(async () => {
      try {
        const [c, t, a, sp] = await Promise.all([
          api.getInboxCards(), api.getHistoryTurns(), api.getActivity(), api.getSpawned(),
        ]);
        setCards(c); setTurns(t); setActivity(a); setSpawned(sp);
      } catch {}
    }, 1500);

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
    <div className="app-canvas flex h-full overflow-hidden">
      <Sidebar view={view} onChange={setView} badge={badge} />

      <main className="flex min-w-0 flex-1 flex-col">
        {view === "sessions" && (
          <SessionsView
            sessions={sessions}
            spawned={spawned}
            onSpawn={async (k, f, p, w) => { await api.spawnSession(k, f, p, w); }}
            onStop={(id) => api.stopSpawned(id)}
            onSend={(id, text) => api.sendPtyInput(id, text)}
          />
        )}
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
        {view === "activity" && <ActivityView report={activity} />}
        {view === "settings" && (
          <SettingsView
            prefs={prefs}
            setPrefs={setPrefs}
            agents={agents}
            hosts={hosts}
            diagnostics={diagnostics}
            iphone={iphone}
          />
        )}
      </main>
    </div>
  );
}
