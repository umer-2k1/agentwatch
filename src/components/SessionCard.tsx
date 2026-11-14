import { useState } from "react";
import { Check, ChevronRight, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { StatusDot } from "./StatusDot";
import { cn } from "@/lib/utils";
import { STATUS_LABEL, vendorOf, type Session } from "@/lib/types";

function age(sec: number) {
  if (sec < 60) return `${sec}s`;
  if (sec < 3600) return `${Math.floor(sec / 60)}m`;
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  return m ? `${h}h ${m}m` : `${h}h`;
}

export function SessionCard({
  session,
  awaiting,
  onAnswer,
}: {
  session: Session;
  awaiting: boolean;
  onAnswer: (id: string, d: "allow" | "deny") => void;
}) {
  const [open, setOpen] = useState(false);
  const v = vendorOf(session.provider);
  const s = session;

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={() => setOpen((o) => !o)}
      onKeyDown={(e) => e.key === "Enter" && setOpen((o) => !o)}
      className={cn(
        "group mb-1.5 cursor-pointer rounded-lg border px-3.5 py-3 transition-all",
        "bg-card/70 backdrop-blur-xl hover:-translate-y-px hover:border-border",
        s.status === "needsYou" && "border-status-needsYou/40 bg-status-needsYou/[0.10]",
        s.status === "dead" && "border-status-dead/35 bg-status-dead/[0.08]",
        (s.status === "idle" || s.status === "unknown") && "opacity-75"
      )}
    >
      <div className="flex items-start gap-3">
        <div className="min-w-0 flex-1">
          <div className="truncate text-[13.5px] font-medium tracking-tight">
            {s.title || s.repo || "Agent"}
          </div>

          <div className="mt-1 flex flex-wrap items-center gap-x-2.5 gap-y-1 text-[11.5px] text-muted-foreground">
            <span className={cn("inline-flex items-center gap-1.5", v.className)}>
              <span className="h-1.5 w-1.5 rounded-[2px]" style={{ background: "hsl(var(--brand))" }} />
              {v.label}
            </span>
            {s.branch && <span>{s.branch}</span>}
            {s.model && <span>{s.model}</span>}
            <span>{s.status === "working" && s.turnAgeS != null ? `${age(s.turnAgeS)} in turn` : `${age(s.quietForS)} ago`}</span>
          </div>

          {awaiting && (
            <div className="mt-2.5 flex items-center gap-2">
              <Button
                size="sm"
                variant="allow"
                onClick={(e) => {
                  e.stopPropagation();
                  onAnswer(s.id, "allow");
                }}
              >
                <Check className="h-3 w-3" /> 1 Allow
              </Button>
              <Button
                size="sm"
                variant="deny"
                onClick={(e) => {
                  e.stopPropagation();
                  onAnswer(s.id, "deny");
                }}
              >
                <X className="h-3 w-3" /> 2 Deny
              </Button>
            </div>
          )}

          {open && (
            <dl className="mt-2.5 grid gap-1 rounded-md bg-muted/50 px-3 py-2 text-[11.5px]">
              <Row k="status" v={`${STATUS_LABEL[s.status]} · ${s.confidence}`} />
              <Row k="why" v={s.evidence?.detail ?? "—"} />
              <Row k="folder" v={s.cwd || "—"} />
              <Row k="pid" v={s.pid ? String(s.pid) : "unknown"} />
            </dl>
          )}
        </div>

        <div className="flex shrink-0 flex-col items-end gap-2 pt-0.5">
          <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
            {age(s.quietForS)}
          </span>
          <StatusDot status={s.status} brand={v.className} size="md" />
          <ChevronRight
            className={cn(
              "h-3 w-3 text-muted-foreground/40 transition-transform",
              open && "rotate-90"
            )}
          />
        </div>
      </div>
    </div>
  );
}

function Row({ k, v }: { k: string; v: string }) {
  return (
    <div className="flex gap-2">
      <dt className="w-12 shrink-0 text-muted-foreground/70">{k}</dt>
      <dd className="min-w-0 truncate text-muted-foreground">{v}</dd>
    </div>
  );
}
