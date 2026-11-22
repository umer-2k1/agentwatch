import { useEffect, useMemo, useState } from "react";
import {
  AlertTriangle, Check, ChevronRight, CornerDownLeft, MessageSquare, Search, X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { StatusDot } from "./StatusDot";
import { cn } from "@/lib/utils";
import { vendorOf, type InboxCard, type TurnRecord } from "@/lib/types";
import * as api from "@/lib/bridge";

export function InboxView({
  cards, turns, onAnswer, onMarkSeen, onMarkAllSeen, onJump,
}: {
  cards: InboxCard[];
  turns: TurnRecord[];
  onAnswer: (id: string, decision: string, label?: string) => void;
  onMarkSeen: (id: string) => void;
  onMarkAllSeen: () => void;
  onJump: (place: string, cwd: string) => void;
}) {
  const [tab, setTab] = useState<"inbox" | "history">("inbox");
  const [selected, setSelected] = useState(0);
  const [query, setQuery] = useState("");
  const [unseenOnly, setUnseenOnly] = useState(false);
  const [now, setNow] = useState(Date.now());

  // tick for the undo countdown + live ages
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 100);
    return () => clearInterval(t);
  }, []);

  const pending = cards.find((c) => c.state === "pending_undo");

  // keyboard: digits, J/K, E, U, R
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA") return;
      if (e.metaKey || e.ctrlKey || e.altKey) return;

      if (e.key === "Tab") {
        e.preventDefault();
        setTab((t) => (t === "inbox" ? "history" : "inbox"));
        return;
      }
      if (e.key === "j" || e.key === "ArrowDown") { setSelected((i) => Math.min(i + 1, cards.length - 1)); return; }
      if (e.key === "k" || e.key === "ArrowUp") { setSelected((i) => Math.max(i - 1, 0)); return; }
      if (e.key === "e" || e.key === "E") {
        const c = cards[selected];
        if (c) onMarkSeen(c.id);
        return;
      }
      if (e.key === "/" ) { e.preventDefault(); (document.getElementById("inbox-search") as HTMLInputElement)?.focus(); return; }
      if (/^[1-9]$/.test(e.key)) {
        const c = cards[selected];
        if (!c || !c.answerable) return;
        const opt = c.options[Number(e.key) - 1];
        if (!opt) return;
        const decision = opt.action === "choose" ? `choose:${opt.number - 1}` : opt.action;
        onAnswer(c.id, decision, `${opt.number}. ${opt.label}`);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [cards, selected, onAnswer, onMarkSeen]);

  const visibleTurns = useMemo(() => {
    let list = turns;
    if (unseenOnly) list = list.filter((t) => !t.seen);
    if (query.trim()) {
      const q = query.toLowerCase();
      list = list.filter((t) =>
        [t.title, t.lastWords, t.folder, t.repo, t.branch, t.provider, t.model]
          .join(" ").toLowerCase().includes(q)
      );
    }
    return list;
  }, [turns, unseenOnly, query]);

  const unseen = turns.filter((t) => !t.seen).length;

  return (
    <div className="flex h-full flex-col">
      {/* header */}
      <div className="flex shrink-0 items-center gap-2 border-b border-white/[0.06] px-3.5 py-2.5">
        <div className="flex rounded-lg bg-black/25 p-0.5">
          <button
            onClick={() => setTab("inbox")}
            className={cn(
              "flex items-center gap-1.5 rounded-full px-3.5 py-1.5 text-[12px] transition-colors",
              tab === "inbox" ? "bg-secondary text-foreground" : "text-muted-foreground hover:text-foreground"
            )}
          >
            Inbox
            {cards.length > 0 && (
              <span className="rounded-full bg-status-needsYou px-1.5 text-[10px] font-bold text-black">
                {cards.length}
              </span>
            )}
          </button>
          <button
            onClick={() => setTab("history")}
            className={cn(
              "rounded-md px-3 py-1 text-[12px] transition-colors",
              tab === "history" ? "bg-secondary text-foreground" : "text-muted-foreground hover:text-foreground"
            )}
          >
            History
          </button>
        </div>

        {tab === "history" && (
          <>
            <div className="relative ml-2 flex-1 max-w-[320px]">
              <Search className="pointer-events-none absolute left-2 top-1/2 h-3 w-3 -translate-y-1/2 text-muted-foreground/60" />
              <input
                id="inbox-search"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Every word must be in the turn…"
                className="h-7 w-full rounded-md border border-border/70 bg-black/25 pl-7 pr-2 text-[11.5px] text-foreground outline-none placeholder:text-muted-foreground/50 focus:border-primary/60"
              />
            </div>
            <button
              onClick={() => setUnseenOnly((v) => !v)}
              className={cn(
                "rounded-md border px-2 py-1 text-[11px]",
                unseenOnly
                  ? "border-primary/60 bg-primary/15 text-primary"
                  : "border-border/70 text-muted-foreground hover:text-foreground"
              )}
            >
              Not seen · {unseen}
            </button>
          </>
        )}

        <div className="ml-auto flex items-center gap-2">
          <Button variant="ghost" size="sm" onClick={onMarkAllSeen}>
            Mark all as seen
          </Button>
        </div>
      </div>

      {/* body */}
      <div className="min-h-0 flex-1 overflow-y-auto px-4 py-3">
        {tab === "inbox" ? (
          cards.length === 0 ? (
            <Empty
              title="Inbox zero"
              body="Nothing needs you. Start Claude Code, Codex or OpenCode and its questions arrive here."
            />
          ) : (
            cards.map((c, i) => (
              <Card
                key={c.id + c.cardType + c.state}
                card={c}
                selected={i === selected}
                now={now}
                onSelect={() => setSelected(i)}
                onAnswer={onAnswer}
                onMarkSeen={onMarkSeen}
                onJump={onJump}
                pendingLabel={pending?.chosenLabel ?? null}
              />
            ))
          )
        ) : visibleTurns.length === 0 ? (
          <Empty title="No turns match" body="Clear the search, or turn off the Not seen filter." />
        ) : (
          visibleTurns.map((t) => <TurnRow key={t.id} turn={t} />)
        )}
      </div>

      {/* legend */}
      <div className="flex shrink-0 items-center gap-3 border-t border-border/70 px-4 py-1.5 text-[10.5px] text-muted-foreground">
        <Key k="1–9" v="answer" /><Key k="J K" v="move" /><Key k="E" v="clear" />
        <Key k="R" v="reply" /><Key k="Tab" v="History" /><Key k="esc" v="undo / close" />
      </div>
    </div>
  );
}

function Card({
  card: c, selected, now, onSelect, onAnswer, onMarkSeen, onJump, pendingLabel,
}: {
  card: InboxCard; selected: boolean; now: number;
  onSelect: () => void;
  onAnswer: (id: string, d: string, label?: string) => void;
  onMarkSeen: (id: string) => void;
  onJump: (place: string, cwd: string) => void;
  pendingLabel: string | null;
}) {
  const v = vendorOf(c.provider);
  const [reply, setReply] = useState("");
  const undoMs = c.undoRemainingMs ?? 0;
  const pct = Math.max(0, Math.min(100, (undoMs / 2000) * 100));

  const isError = c.cardType === "auth" || c.cardType === "balance";

  return (
    <div
      onClick={onSelect}
      className={cn(
        "mb-2.5 cursor-pointer rounded-[18px] border px-4 py-3.5 transition-all",
        "border-white/[0.07] bg-white/[0.035] hover:border-white/[0.12] hover:bg-white/[0.055]",
        selected && "border-primary/50",
        c.cardType === "permission" &&
          "border-status-needsYou/35 bg-gradient-to-b from-status-needsYou/[0.11] to-status-needsYou/[0.045]",
        isError && "border-status-dead/35 bg-gradient-to-b from-status-dead/[0.09] to-status-dead/[0.03]"
      )}
    >
      {/* card header */}
      <div className="flex items-center gap-2 text-[11.5px]">
        <StatusDot
          status={isError ? "dead" : c.cardType === "finished" ? "done" : "needsYou"}
          brand={v.className}
          size="sm"
        />
        <span className="truncate font-medium text-foreground">{c.title}</span>
        <span className="ml-auto flex shrink-0 items-center gap-2 text-muted-foreground">
          <span className="truncate max-w-[130px] opacity-80">{c.repo}</span>
          <span className="rounded-md bg-white/[0.07] px-1.5 py-px text-[9.5px]">{c.place}</span>
          <span className="w-8 text-right font-mono text-[10px]">{age(c.ageS)}</span>
        </span>
      </div>

      {/* body */}
      {c.header && (
        <div className="mt-2.5 text-[10px] font-semibold uppercase tracking-[0.08em] text-muted-foreground/80">
          {c.header}
        </div>
      )}
      {c.command && (
        <pre className="mt-1.5 overflow-x-auto rounded-md bg-black/40 px-2.5 py-2 font-mono text-[11.5px] text-foreground/90">
          {c.command}
        </pre>
      )}
      {c.question && (
        <div className="mt-2 text-[12.5px] font-medium text-foreground">{c.question}</div>
      )}
      {!c.question && !c.command && (
        <div className="mt-1.5 text-[12px] leading-relaxed text-muted-foreground">{c.body}</div>
      )}

      {/* options */}
      {c.options.length > 0 && (
        <div className="mt-2 space-y-1">
          {c.options.map((o) => (
            <button
              key={o.number}
              disabled={!c.answerable || c.state === "pending_undo"}
              onClick={(e) => {
                e.stopPropagation();
                const decision = o.action === "choose" ? `choose:${o.number - 1}` : o.action;
                onAnswer(c.id, decision, `${o.number}. ${o.label}`);
              }}
              className={cn(
                "flex w-full items-center gap-3 rounded-xl border px-3.5 py-2.5 text-left text-[12.5px] transition-colors",
                "border-white/[0.08] bg-white/[0.03] hover:border-primary/45 hover:bg-primary/[0.08]",
                c.state === "pending_undo" && "opacity-60"
              )}
            >
              <span className="grid h-5 w-5 shrink-0 place-items-center rounded bg-white/10 font-mono text-[10px]">
                {o.number}
              </span>
              <span className="font-medium text-foreground">{o.label}</span>
              {o.description && (
                <span className="ml-auto text-[10.5px] text-muted-foreground">{o.description}</span>
              )}
            </button>
          ))}
        </div>
      )}

      {/* finished card actions */}
      {c.cardType === "finished" && (
        <div className="mt-2.5 space-y-1.5">
          <div className="flex items-center gap-2 rounded-xl border border-white/[0.08] bg-white/[0.03] px-3 py-2">
            <MessageSquare className="h-3 w-3 text-muted-foreground" />
            <input
              value={reply}
              onChange={(e) => setReply(e.target.value)}
              placeholder={`Reply to ${c.title}…`}
              className="flex-1 bg-transparent text-[11.5px] outline-none placeholder:text-muted-foreground/50"
            />
            <kbd className="rounded bg-white/8 px-1 text-[9px] text-muted-foreground">R</kbd>
          </div>
          <div className="flex gap-2">
            <Button size="sm" variant="outline" onClick={(e) => { e.stopPropagation(); onMarkSeen(c.id); }}>
              <Check className="h-3 w-3" /> Mark seen
            </Button>
            <Button size="sm" variant="ghost">
              Ask OpenCode to review
            </Button>
          </div>
        </div>
      )}

      {/* elsewhere / error actions */}
      {(c.cardType === "elsewhere" || isError) && (
        <div className="mt-2.5 flex items-center gap-2">
          <Button size="sm" variant="outline" onClick={(e) => { e.stopPropagation(); onJump(c.place, c.repo); }}>
            <CornerDownLeft className="h-3 w-3" /> Open in {c.place}
          </Button>
          {isError && (
            <span className="flex items-center gap-1 text-[11px] text-status-dead">
              <AlertTriangle className="h-3 w-3" /> {c.cardType.toUpperCase()} — never shown as "Finished."
            </span>
          )}
        </div>
      )}

      {/* 2-second undo bar (Figures 54–55) */}
      {c.state === "pending_undo" && (
        <div className="mt-3">
          <div className="flex items-center justify-between text-[11px]">
            <span className="text-foreground">
              Sending “{c.chosenLabel ?? pendingLabel}”
            </span>
            <span className="flex items-center gap-1 text-muted-foreground">
              <kbd className="rounded bg-white/10 px-1 text-[9px]">esc</kbd> undo
            </span>
          </div>
          <div className="mt-1.5 h-1 overflow-hidden rounded-full bg-white/10">
            <div
              className="h-full rounded-full bg-status-working transition-[width] duration-100 ease-linear"
              style={{ width: `${pct}%` }}
            />
          </div>
        </div>
      )}
    </div>
  );
}

function TurnRow({ turn: t }: { turn: TurnRecord }) {
  const v = vendorOf(t.provider);
  const isError = t.outcome === "error";
  return (
    <div className="mb-1.5 rounded-xl border border-border/70 bg-card/50 p-3">
      <div className="flex items-center gap-2 text-[11.5px]">
        <StatusDot status={isError ? "dead" : t.seen ? "idle" : "done"} brand={v.className} size="sm" />
        <span className="font-mono text-[10.5px] text-muted-foreground">
          {new Date(t.finishedMs).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
        </span>
        <span className="truncate font-medium text-foreground">{t.title}</span>
        {!t.seen && !isError && (
          <span className="rounded-full border border-status-done/40 px-1.5 text-[9.5px] text-status-done">Not seen</span>
        )}
        {isError && (
          <span className="rounded-full border border-status-dead/40 px-1.5 text-[9.5px] text-status-dead">
            {t.errorKind ?? "error"}
          </span>
        )}
        <span className="ml-auto shrink-0 text-muted-foreground">
          {duration(t.durationS)} · {t.repo} ⎇ {t.branch}
        </span>
      </div>
      <div className="mt-1.5 line-clamp-2 text-[11.5px] text-muted-foreground">{t.lastWords || "Finished."}</div>
    </div>
  );
}

function Empty({ title, body }: { title: string; body: string }) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-1 text-center">
      <p className="text-[13px] font-medium text-foreground/80">{title}</p>
      <p className="max-w-[360px] text-[12px] text-muted-foreground">{body}</p>
    </div>
  );
}

function Key({ k, v }: { k: string; v: string }) {
  return (
    <span className="flex items-center gap-1">
      <kbd className="rounded bg-white/8 px-1 font-mono text-[9.5px]">{k}</kbd> {v}
    </span>
  );
}

function age(s: number) {
  if (s < 60) return `${s}s`;
  if (s < 3600) return `${Math.floor(s / 60)}m`;
  return `${Math.floor(s / 3600)}h`;
}

function duration(s: number) {
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  return m < 60 ? `${m}m` : `${Math.floor(m / 60)}h ${m % 60}m`;
}
