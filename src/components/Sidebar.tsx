import { Activity, Inbox, LayoutGrid, Settings, Sparkles } from "lucide-react";
import { cn } from "@/lib/utils";

export type View = "sessions" | "inbox" | "activity" | "settings";

const ITEMS: { id: View; label: string; icon: typeof Inbox }[] = [
  { id: "sessions", label: "Sessions", icon: LayoutGrid },
  { id: "inbox", label: "Inbox", icon: Inbox },
  { id: "activity", label: "Activity", icon: Activity },
  { id: "settings", label: "Settings", icon: Settings },
];

export function Sidebar({
  view, onChange, badge,
}: { view: View; onChange: (v: View) => void; badge: number }) {
  return (
    <nav className="flex w-[188px] shrink-0 flex-col border-r border-border/70 bg-card/40 px-2.5 py-3">
      <div className="mb-4 flex items-center gap-2 px-2">
        <span className="h-2 w-2 rounded-full bg-primary shadow-[0_0_0_3px_hsl(var(--primary)/0.2)]" />
        <span className="text-[13px] font-semibold tracking-tight">AgentWatch</span>
      </div>

      {ITEMS.map(({ id, label, icon: Icon }) => (
        <button
          key={id}
          onClick={() => onChange(id)}
          className={cn(
            "mb-0.5 flex items-center gap-2.5 rounded-md px-2.5 py-2 text-left text-[12.5px] transition-colors",
            view === id
              ? "bg-secondary/80 font-medium text-foreground"
              : "text-muted-foreground hover:bg-secondary/40 hover:text-foreground"
          )}
        >
          <Icon className="h-3.5 w-3.5" />
          <span className="flex-1">{label}</span>
          {id === "inbox" && badge > 0 && (
            <span className="rounded-full bg-status-needsYou px-1.5 py-px text-[10px] font-semibold text-black">
              {badge}
            </span>
          )}
        </button>
      ))}

      <div className="mt-auto space-y-1.5 rounded-lg border border-border/60 bg-black/20 p-2.5">
        <div className="flex items-center gap-1.5 text-[10.5px] font-semibold uppercase tracking-wider text-muted-foreground/70">
          <Sparkles className="h-3 w-3" /> Shortcuts
        </div>
        <Row k="Inbox" v="⌃⌥I" />
        <Row k="Ask agent" v="⌥⌥" />
        <Row k="Hover card" v="⌃⌥H" />
      </div>
    </nav>
  );
}

function Row({ k, v }: { k: string; v: string }) {
  return (
    <div className="flex items-center justify-between text-[11px] text-muted-foreground">
      <span>{k}</span>
      <kbd className="rounded bg-white/8 px-1.5 py-px font-mono text-[10px] text-foreground/80">{v}</kbd>
    </div>
  );
}
