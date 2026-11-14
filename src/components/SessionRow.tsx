import { useState } from "react";
import { Check, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { StatusDot } from "./StatusDot";
import { ProviderGlyph } from "./ProviderGlyph";
import { cn } from "@/lib/utils";
import { STATUS_LABEL, vendorOf, type Session } from "@/lib/types";

export function age(sec: number) {
  if (sec < 60) return `${sec}s`;
  if (sec < 3600) return `${Math.floor(sec / 60)}m`;
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  return m ? `${h}h ${m}m` : `${h}h`;
}

/**
 * One agent, as a flat row inside the panel — not a card.
 *
 * AgentDock's list is one glass surface with hairlines, so the eye reads it as a single
 * object. Separately bordered cards (what we had) read as a stack of unrelated boxes.
 */
export function SessionRow({
  session: s,
  awaiting,
  onAnswer,
}: {
  session: Session;
  awaiting: boolean;
  onAnswer: (id: string, decision: "allow" | "deny") => void;
}) {
  const [open, setOpen] = useState(false);
  const v = vendorOf(s.provider);
  const needsYou = s.status === "needsYou";
  const dead = s.status === "dead";

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={() => setOpen((o) => !o)}
      onKeyDown={(e) => e.key === "Enter" && setOpen((o) => !o)}
      className={cn(
        "group relative cursor-pointer border-t border-white/[0.045] px-4 py-3 transition-colors first:border-t-0",
        "hover:bg-white/[0.035]",
        needsYou && "bg-status-needsYou/[0.07] hover:bg-status-needsYou/[0.10]",
        dead && "bg-status-dead/[0.05] hover:bg-status-dead/[0.08]"
      )}
    >
      {/* left accent bar — AgentDock marks attention at the edge, not with a border */}
      {needsYou && <span className="absolute inset-y-0 left-0 w-[2px] bg-status-needsYou" />}

      <div className="flex items-start gap-2.5">
        <span className={cn("mt-[3px]", v.className)}>
          <ProviderGlyph provider={s.provider} />
        </span>

        <div className="min-w-0 flex-1">
          <div className="flex items-baseline gap-2">
            <span className="truncate text-[13px] font-medium leading-5 tracking-[-0.01em] text-foreground">
              {s.title || s.repo || "Agent"}
            </span>
          </div>

          <div className="mt-0.5 flex items-center gap-1.5 truncate text-[11px] leading-4 text-muted-foreground">
            <span
              className={cn(
                "font-medium",
                needsYou && "text-status-needsYou",
                dead && "text-status-dead",
                s.status === "working" && "text-status-working",
                s.status === "done" && "text-status-done"
              )}
            >
              {STATUS_LABEL[s.status]}
            </span>
            <span className="opacity-30">·</span>
            <span className="truncate">{s.repo}</span>
            {s.branch && (
              <>
                <span className="opacity-30">·</span>
                <span className="truncate">{s.branch}</span>
              </>
            )}
            {s.model && (
              <>
                <span className="opacity-30">·</span>
                <span className="truncate">{s.model}</span>
              </>
            )}
          </div>

          {awaiting && (
            <div className="mt-2 flex items-center gap-1.5">
              <Button
                size="sm"
                variant="allow"
                onClick={(e) => {
                  e.stopPropagation();
                  onAnswer(s.id, "allow");
                }}
              >
                <Check className="h-3 w-3" /> Allow
                <kbd className="ml-1 rounded bg-status-done/15 px-1 text-[9px]">1</kbd>
              </Button>
              <Button
                size="sm"
                variant="deny"
                onClick={(e) => {
                  e.stopPropagation();
                  onAnswer(s.id, "deny");
                }}
              >
                <X className="h-3 w-3" /> Deny
                <kbd className="ml-1 rounded bg-status-dead/15 px-1 text-[9px]">2</kbd>
              </Button>
            </div>
          )}

          {open && (
            <dl className="mt-2 grid gap-0.5 rounded-md bg-black/25 px-2.5 py-2 text-[11px]">
              <Row k="why" v={s.evidence?.detail ?? "—"} />
              <Row k="folder" v={s.cwd || "—"} />
              <Row k="pid" v={s.pid ? String(s.pid) : "unknown"} />
              <Row k="confidence" v={s.confidence} />
            </dl>
          )}
        </div>

        <div className="flex shrink-0 items-center gap-2 pt-1">
          <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground/80">
            {age(s.quietForS)}
          </span>
          <StatusDot status={s.status} brand={v.className} size="md" />
        </div>
      </div>
    </div>
  );
}

function Row({ k, v }: { k: string; v: string }) {
  return (
    <div className="flex gap-2">
      <dt className="w-[68px] shrink-0 text-muted-foreground/60">{k}</dt>
      <dd className="min-w-0 truncate text-muted-foreground">{v}</dd>
    </div>
  );
}
