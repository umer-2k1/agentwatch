import { StatusDot } from "./StatusDot";
import { vendorOf, type Session } from "@/lib/types";

/** The ambient strip: one bare dot per agent, no labels. The list carries the detail. */
export function DotRail({ sessions }: { sessions: Session[] }) {
  if (!sessions.length) return null;
  return (
    <div className="flex shrink-0 items-center gap-2.5 overflow-x-auto border-b px-3.5 py-2">
      {sessions.map((s) => {
        const v = vendorOf(s.provider);
        return (
          <span key={s.id} title={`${s.repo || s.title} — ${s.status}`}>
            <StatusDot status={s.status} brand={v.className} size="md" />
          </span>
        );
      })}
    </div>
  );
}
