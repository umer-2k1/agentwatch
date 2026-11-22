import { useState } from "react";
import { Play, Plus, Square, Terminal } from "lucide-react";
import { Button } from "@/components/ui/button";
import { StatusDot } from "./StatusDot";
import { cn } from "@/lib/utils";
import { STATUS_LABEL, vendorOf, type Session, type SpawnedView } from "@/lib/types";

const FILTERS = ["all", "needsYou", "working", "done", "dead"] as const;
type Filter = (typeof FILTERS)[number];

export function SessionsView({
  sessions, spawned, onSpawn, onStop, onSend,
}: {
  sessions: Session[];
  spawned: SpawnedView[];
  onSpawn: (kind: string, folder: string, prompt: string, worktree: boolean) => void;
  onStop: (id: string) => void;
  onSend: (id: string, text: string) => void;
}) {
  const [filter, setFilter] = useState<Filter>("all");
  const [open, setOpen] = useState(false);
  const [kind, setKind] = useState("claude-code");
  const [folder, setFolder] = useState("~/");
  const [prompt, setPrompt] = useState("");
  const [worktree, setWorktree] = useState(false);

  const list = sessions.filter((s) => filter === "all" || s.status === filter);
  const cols = list.length <= 1 ? 1 : list.length <= 4 ? 2 : 3;

  return (
    <div className="flex h-full flex-col">
      <div className="flex shrink-0 items-center gap-2 border-b border-border/70 px-4 py-2">
        <div className="flex gap-1">
          {FILTERS.map((f) => (
            <button
              key={f}
              onClick={() => setFilter(f)}
              className={cn(
                "rounded-md px-2.5 py-1 text-[11px] capitalize",
                filter === f ? "bg-secondary text-foreground" : "text-muted-foreground hover:text-foreground"
              )}
            >
              {f === "all" ? "All" : STATUS_LABEL[f as keyof typeof STATUS_LABEL] ?? f}
            </button>
          ))}
        </div>
        <Button size="sm" variant="default" className="ml-auto" onClick={() => setOpen((o) => !o)}>
          <Plus className="h-3 w-3" /> New session
        </Button>
      </div>

      {open && (
        <div className="flex shrink-0 flex-wrap items-end gap-2 border-b border-border/70 bg-black/20 px-4 py-2.5">
          <label className="flex flex-col gap-1 text-[10px] uppercase tracking-wider text-muted-foreground/70">
            Agent
            <select value={kind} onChange={(e) => setKind(e.target.value)}
              className="rounded border border-border/70 bg-black/30 px-2 py-1 text-[11.5px] normal-case tracking-normal text-foreground outline-none">
              <option value="claude-code">Claude Code</option>
              <option value="opencode">OpenCode</option>
              <option value="codex">Codex</option>
              <option value="shell">Shell</option>
            </select>
          </label>
          <label className="flex min-w-[180px] flex-1 flex-col gap-1 text-[10px] uppercase tracking-wider text-muted-foreground/70">
            Folder
            <input value={folder} onChange={(e) => setFolder(e.target.value)}
              className="rounded border border-border/70 bg-black/30 px-2 py-1 font-mono text-[11px] normal-case tracking-normal text-foreground outline-none" />
          </label>
          <label className="flex min-w-[220px] flex-[2] flex-col gap-1 text-[10px] uppercase tracking-wider text-muted-foreground/70">
            First prompt
            <input value={prompt} onChange={(e) => setPrompt(e.target.value)} placeholder="Run the tests and fix what fails"
              className="rounded border border-border/70 bg-black/30 px-2 py-1 text-[11.5px] normal-case tracking-normal text-foreground outline-none placeholder:text-muted-foreground/50" />
          </label>
          <label className="flex items-center gap-1.5 pb-1 text-[11px] text-muted-foreground">
            <input type="checkbox" checked={worktree} onChange={(e) => setWorktree(e.target.checked)} />
            New git worktree
          </label>
          <Button size="sm" onClick={() => { onSpawn(kind, folder, prompt, worktree); setOpen(false); }}>
            <Play className="h-3 w-3" /> Spawn
          </Button>
        </div>
      )}

      <div className="min-h-0 flex-1 overflow-y-auto p-3">
        {list.length === 0 ? (
          <div className="grid h-full place-items-center text-[13px] text-muted-foreground">No sessions match this filter.</div>
        ) : (
          <div className={cn("grid gap-2.5", cols === 1 && "grid-cols-1", cols === 2 && "grid-cols-2", cols === 3 && "grid-cols-3")}>
            {list.map((s) => {
              const v = vendorOf(s.provider);
              const sp = spawned.find((x) => x.id === s.id);
              return (
                <div key={s.id} className="flex flex-col rounded-xl border border-border/70 bg-card/60">
                  <div className="flex items-center gap-2 border-b border-border/50 px-2.5 py-1.5">
                    <StatusDot status={s.status} brand={v.className} size="sm" />
                    <span className="truncate text-[11.5px] font-medium">{s.title}</span>
                    <span className={cn("ml-auto shrink-0 rounded-full px-1.5 py-px text-[9.5px]",
                      s.status === "needsYou" ? "bg-status-needsYou/20 text-status-needsYou" :
                      s.status === "working" ? "bg-status-working/20 text-status-working" :
                      s.status === "dead" ? "bg-status-dead/20 text-status-dead" :
                      "bg-white/8 text-muted-foreground")}>
                      {STATUS_LABEL[s.status]}
                    </span>
                  </div>

                  <div className="flex-1 overflow-y-auto px-2.5 py-2 font-mono text-[10.5px] leading-relaxed text-muted-foreground/90" style={{ maxHeight: 190 }}>
                    {sp ? (
                      sp.lines.slice(-40).map((l, i) => <div key={i} className="whitespace-pre-wrap break-words">{l}</div>)
                    ) : (
                      <>
                        <div>{s.repo} ⎇ {s.branch} · {s.model || v.label}</div>
                        <div className="mt-1 text-muted-foreground/70">{s.lastWords || s.evidence.detail}</div>
                        {(s.editedFiles ?? []).slice(0, 4).map((f) => (
                          <div key={f.path} className="mt-0.5 text-[10px] text-muted-foreground/60">± {f.path.split("/").pop()}</div>
                        ))}
                      </>
                    )}
                  </div>

                  <div className="border-t border-border/50 px-2 py-1.5">
                    <form
                      onSubmit={(e) => {
                        e.preventDefault();
                        const el = (e.currentTarget.elements.namedItem("msg") as HTMLInputElement);
                        if (el.value.trim()) { onSend(s.id, el.value.trim()); el.value = ""; }
                      }}
                      className="flex items-center gap-1.5"
                    >
                      <Terminal className="h-3 w-3 text-muted-foreground/60" />
                      <input name="msg" placeholder="Send to agent…"
                        className="flex-1 bg-transparent text-[11px] outline-none placeholder:text-muted-foreground/40" />
                      {sp?.running && (
                        <button type="button" onClick={() => onStop(s.id)} className="text-status-dead/80 hover:text-status-dead">
                          <Square className="h-3 w-3" />
                        </button>
                      )}
                    </form>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
