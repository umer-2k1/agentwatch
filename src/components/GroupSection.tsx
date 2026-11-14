import { Folder } from "lucide-react";
import { SessionRow } from "./SessionRow";
import type { Session } from "@/lib/types";

export function GroupSection({
  repo,
  sessions,
  awaiting,
  onAnswer,
  first,
}: {
  repo: string;
  sessions: Session[];
  awaiting: Set<string>;
  onAnswer: (id: string, d: "allow" | "deny") => void;
  first?: boolean;
}) {
  return (
    <div className={first ? "" : "mt-1"}>
      <div className="flex items-center gap-1.5 px-4 py-1.5">
        <Folder className="h-2.5 w-2.5 text-muted-foreground/50" />
        <span className="text-[10.5px] font-semibold uppercase tracking-[0.06em] text-muted-foreground/70">
          {repo}
        </span>
        <span className="text-[10px] text-muted-foreground/40">{sessions.length}</span>
      </div>
      <div>
        {sessions.map((s) => (
          <SessionRow
            key={s.id}
            session={s}
            awaiting={awaiting.has(s.id)}
            onAnswer={onAnswer}
          />
        ))}
      </div>
    </div>
  );
}
