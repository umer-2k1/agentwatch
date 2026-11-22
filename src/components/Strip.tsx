import { useEffect, useState } from "react";
import { getSessions, onSessions, inTauri } from "@/lib/bridge";
import { vendorOf, type Session } from "@/lib/types";
import { cn } from "@/lib/utils";

/**
 * The edge strip.
 *
 * A 44 pt transparent column pinned to the right edge of the screen. Each agent is one dot
 * inside a dark pill; hovering grows the pill slightly, clicking reveals the window focused
 * on that agent. This is the surface the product is named for — everything else is detail.
 */
export function Strip() {
  const [sessions, setSessions] = useState<Session[]>([]);

  useEffect(() => {
    let stop: undefined | (() => void);
    (async () => {
      setSessions(await getSessions());
      stop = await onSessions(setSessions);
    })();
    const t = setInterval(async () => {
      try { setSessions(await getSessions()); } catch {}
    }, 1500);
    return () => { stop?.(); clearInterval(t); };
  }, []);

  async function open(id: string) {
    if (!inTauri()) return;
    await window.__TAURI__!.core.invoke("focus_agent", { sessionId: id });
  }

  // The strip is ambient: only work that is happening or waiting on you earns a dot.
  // Idle/unknown sessions stay in the window; putting 29 dots on the desktop is noise.
  const visible = sessions
    .filter((s) => s.status === "needsYou" || s.status === "working" || s.status === "done" || s.status === "dead")
    .slice(0, 8);

  if (!visible.length) return <div className="h-screen w-full bg-transparent" />;

  return (
    <div className="flex h-screen w-full flex-col items-center justify-center gap-1.5 bg-transparent">
      {visible.map((s) => {
        const v = vendorOf(s.provider);
        return (
          <button
            key={s.id}
            onClick={() => open(s.id)}
            title={`${s.repo || s.title} — ${s.status}`}
            className={cn(
              "group flex h-[22px] w-[22px] items-center justify-center rounded-full",
              "bg-black/55 backdrop-blur-md transition-all hover:h-[24px] hover:w-[24px]",
              "ring-1 ring-white/10"
            )}
          >
            <span
              className={cn(
                "h-2.5 w-2.5 rounded-full",
                s.status === "working" && "bg-status-working",
                s.status === "needsYou" && "bg-status-needsYou animate-pulse-ring",
                s.status === "done" && "bg-status-done",
                s.status === "idle" && "bg-status-idle/70",
                s.status === "dead" && "bg-status-dead",
                s.status === "unknown" && "border-2 border-dashed border-status-unknown bg-transparent",
                v.className
              )}
              style={{ boxShadow: "0 0 0 1.5px hsl(var(--brand, transparent) / 0.8)" }}
            />
          </button>
        );
      })}
      {sessions.filter((s) => s.status === "needsYou" || s.status === "working").length > 8 && (
        <span className="mt-1 rounded-full bg-black/55 px-1.5 py-0.5 text-[9px] text-white/60">
          +{sessions.filter((s) => s.status === "needsYou" || s.status === "working").length - 8}
        </span>
      )}
    </div>
  );
}
