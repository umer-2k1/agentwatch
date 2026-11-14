import { Folder } from "lucide-react";
import { SessionCard } from "./SessionCard";
import type { Session } from "@/lib/types";

export function GroupSection({
  repo,
  sessions,
  awaiting,
  onAnswer,
}: {
  repo: string;
  sessions: Session[];
  awaiting: Set<string>;
  onAnswer: (id: string, d: "allow" | "deny") => void;
}) {
  return (
    <section className="mb-4">
      <div className="mb-1.5 flex items-center gap-1.5 px-1.5 text-[11px] font-semibold text-muted-foreground">
        <Folder className="h-3 w-3 opacity-60" />
        <span>{repo}</span>
        <span className="rounded bg-muted px-1.5 py-px text-[10px] text-muted-foreground/80">
          {sessions.length}
        </span>
      </div>
      {sessions.map((s) => (
        <SessionCard
          key={s.id}
          session={s}
          awaiting={awaiting.has(s.id)}
          onAnswer={onAnswer}
        />
      ))}
    </section>
  );
}
