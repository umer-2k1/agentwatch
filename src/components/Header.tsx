import { Settings2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { STATUS_LABEL, STATUS_ORDER, type Session } from "@/lib/types";

export function Header({
  sessions,
  onSettings,
}: {
  sessions: Session[];
  onSettings: () => void;
}) {
  const counts = sessions.reduce<Record<string, number>>((acc, s) => {
    acc[s.status] = (acc[s.status] ?? 0) + 1;
    return acc;
  }, {});

  return (
    <header className="glass flex h-11 shrink-0 items-center justify-between border-b px-3.5">
      <div className="flex items-center gap-2">
        <span className="h-2 w-2 rounded-full bg-primary shadow-[0_0_0_3px_hsl(var(--primary)/0.22)]" />
        <span className="text-[13px] font-semibold tracking-tight">AgentWatch</span>
      </div>

      <div className="flex items-center gap-4">
        <div className="flex items-center gap-3 text-[11.5px] text-muted-foreground">
          {STATUS_ORDER.filter((s) => counts[s]).map((s) => (
            <span key={s} className="whitespace-nowrap">
              <span className="font-semibold text-foreground">{counts[s]}</span> {STATUS_LABEL[s]}
            </span>
          ))}
        </div>
        <Button variant="ghost" size="icon" onClick={onSettings} aria-label="Appearance">
          <Settings2 className="h-3.5 w-3.5" />
        </Button>
      </div>
    </header>
  );
}
