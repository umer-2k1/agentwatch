import { useEffect, useState } from "react";
import { getSessions, onSessions, inTauri } from "@/lib/bridge";
import { vendorOf, type Session } from "@/lib/types";
import { cn } from "@/lib/utils";

/**
 * The edge strip — a compact vertical capsule of status dots (Blueprint §4.1, Figure 25b).
 *
 * Idle/unknown sessions stay in the window; only live work earns a dot.
 */
export function Strip() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [hovered, setHovered] = useState(false);

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

  const visible = sessions
    .filter((s) => s.status === "needsYou" || s.status === "working" || s.status === "done" || s.status === "dead")
    .slice(0, 8);

  if (!visible.length) return <div className="h-screen w-full bg-transparent" />;

  return (
    <div className="flex h-screen w-full items-center justify-center bg-transparent">
      <div
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        className={cn(
          "flex flex-col items-center gap-[5px] rounded-full px-[5px] py-[7px]",
          "bg-black/70 ring-1 ring-white/10 backdrop-blur-md transition-all",
          hovered && "bg-black/85 ring-white/20"
        )}
      >
        {visible.map((s) => {
          const v = vendorOf(s.provider);
          return (
            <button
              key={s.id}
              onClick={() => open(s.id)}
              title={`${s.repo || s.title} — ${s.status}`}
              className="grid h-3.5 w-3.5 place-items-center rounded-full transition-transform hover:scale-125"
            >
              <span
                className={cn(
                  "h-2.5 w-2.5 rounded-full",
                  s.status === "working" && "bg-status-working",
                  s.status === "needsYou" && "bg-status-needsYou animate-pulse-ring",
                  s.status === "done" && "bg-status-done",
                  s.status === "dead" && "bg-status-dead",
                  v.className
                )}
                style={{ boxShadow: "0 0 0 1.2px hsl(var(--brand, rgba(255,255,255,.35)) / 0.9)" }}
              />
            </button>
          );
        })}
      </div>
    </div>
  );
}
