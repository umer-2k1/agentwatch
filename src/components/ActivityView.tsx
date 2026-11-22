import { useMemo, useState } from "react";
import { ChevronLeft, ChevronRight, Play } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { ActivityReport } from "@/lib/types";

const HOURS_LABEL = ["12a", "2a", "4a", "6a", "8a", "10a", "12p", "2p", "4p", "6p", "8p", "10p"];

export function ActivityView({ report }: { report: ActivityReport | null }) {
  const [selected, setSelected] = useState(report ? report.days.length - 1 : 0);
  const [metric, setMetric] = useState<"hours" | "turns">("hours");
  const [range, setRange] = useState<"day" | "week" | "month">("day");

  const day = report?.days[selected];
  const maxHours = useMemo(
    () => Math.max(1, ...(report?.days.map((d) => (metric === "hours" ? d.hours : d.turns)) ?? [1])),
    [report, metric]
  );

  if (!report) {
    return (
      <div className="grid h-full place-items-center text-[13px] text-muted-foreground">No activity recorded yet.</div>
    );
  }

  return (
    <div className="h-full overflow-y-auto px-5 py-4">
      {/* header summary */}
      <div className="mb-4">
        <div className="text-[10.5px] font-semibold uppercase tracking-[0.08em] text-muted-foreground/70">
          The last 12 months
        </div>
        <div className="mt-1 flex flex-wrap items-baseline gap-x-2 text-[13px] text-muted-foreground">
          <span className="text-[26px] font-semibold tracking-tight text-foreground">
            {report.totalHours} hours
          </span>
          <span>of agent work ·</span>
          <span className="text-foreground">{report.totalTurns.toLocaleString()} turns</span>
          <span>in {report.totalSessions} sessions ·</span>
          <span className="text-foreground">{report.activeDays} of 365 days</span>
          <span>· {(report.totalTokens / 1_000_000).toFixed(0)}M tokens ·</span>
          <span className="text-foreground">≈ ${report.totalCostUsd.toFixed(2)}</span>
          <span>· {report.currentStreak}-day streak · mostly {report.topModel}</span>
        </div>
      </div>

      {/* heatmap */}
      <div className="mb-3 rounded-xl border border-border/70 bg-card/50 p-3">
        <div className="mb-2 flex items-center gap-2">
          <div className="flex rounded-md bg-black/30 p-0.5">
            {(["hours", "turns"] as const).map((m) => (
              <button
                key={m}
                onClick={() => setMetric(m)}
                className={cn(
                  "rounded px-2 py-0.5 text-[10.5px] capitalize",
                  metric === m ? "bg-secondary text-foreground" : "text-muted-foreground"
                )}
              >
                Agent {m}
              </button>
            ))}
          </div>
          <span className="text-[10.5px] text-muted-foreground">Click a day to replay it · ← → step through days</span>
          <span className="ml-auto flex items-center gap-1 text-[10px] text-muted-foreground">
            Less
            <span className="h-2.5 w-2.5 rounded-sm bg-white/8" />
            <span className="h-2.5 w-2.5 rounded-sm bg-primary/30" />
            <span className="h-2.5 w-2.5 rounded-sm bg-primary/60" />
            <span className="h-2.5 w-2.5 rounded-sm bg-primary" />
            More
          </span>
        </div>

        <div className="flex gap-[3px] overflow-x-auto pb-1">
          {report.days.map((d, i) => {
            const v = metric === "hours" ? d.hours : d.turns;
            const level = v === 0 ? 0 : Math.min(3, Math.ceil((v / maxHours) * 3));
            return (
              <button
                key={d.date}
                onClick={() => setSelected(i)}
                title={`${d.label} · ${d.hours}h · ${d.turns} turns`}
                className={cn(
                  "h-3 w-3 shrink-0 rounded-sm transition-colors",
                  level === 0 && "bg-white/[0.06]",
                  level === 1 && "bg-primary/30",
                  level === 2 && "bg-primary/60",
                  level === 3 && "bg-primary",
                  i === selected && "ring-2 ring-foreground/70"
                )}
              />
            );
          })}
        </div>
      </div>

      {/* selected day */}
      {day && (
        <>
          <div className="mb-2 flex items-center gap-3">
            <div>
              <div className="text-[15px] font-semibold tracking-tight">{day.label}</div>
              <div className="text-[11.5px] text-muted-foreground">
                {day.turns} turns · {day.hours}h of agent work · {day.sessions} sessions · up to{" "}
                {day.maxConcurrent} at once
              </div>
            </div>
            <div className="ml-auto flex items-center gap-1.5">
              <div className="flex rounded-md bg-black/30 p-0.5">
                {(["day", "week", "month"] as const).map((r) => (
                  <button
                    key={r}
                    onClick={() => setRange(r)}
                    className={cn(
                      "rounded px-2 py-0.5 text-[10.5px] capitalize",
                      range === r ? "bg-secondary text-foreground" : "text-muted-foreground"
                    )}
                  >
                    {r}
                  </button>
                ))}
              </div>
              <Button variant="ghost" size="icon" onClick={() => setSelected((i) => Math.max(0, i - 1))}>
                <ChevronLeft className="h-3.5 w-3.5" />
              </Button>
              <Button variant="ghost" size="icon" onClick={() => setSelected((i) => Math.min(report.days.length - 1, i + 1))}>
                <ChevronRight className="h-3.5 w-3.5" />
              </Button>
              <Button size="sm" variant="default">
                <Play className="h-3 w-3" /> Replay
              </Button>
            </div>
          </div>

          <div className="mb-3 rounded-xl border border-border/70 bg-card/50 p-3">
            {/* hour axis */}
            <div className="mb-1.5 flex pl-[110px] text-[9.5px] text-muted-foreground/70">
              {HOURS_LABEL.map((h) => (
                <span key={h} className="flex-1">{h}</span>
              ))}
            </div>

            {/* at-once line */}
            <div className="mb-2 flex items-center gap-2">
              <div className="w-[110px] shrink-0 text-[10.5px] leading-tight text-muted-foreground">
                At once<br /><span className="text-[9.5px] opacity-70">agents working</span>
              </div>
              <div className="relative h-8 flex-1">
                <div className="absolute inset-x-0 bottom-0 h-6 rounded bg-primary/[0.07]" />
                <svg viewBox="0 0 240 24" preserveAspectRatio="none" className="absolute inset-0 h-6 w-full">
                  <polyline
                    points={day.hourly.map((v, h) => `${(h / 23) * 240},${24 - Math.min(24, v * 2)}`).join(" ")}
                    fill="none" stroke="hsl(var(--status-working))" strokeWidth="1.4"
                  />
                </svg>
                <span className="absolute left-[52%] top-0 rounded bg-primary px-1 text-[9px] font-semibold text-white">
                  {day.maxConcurrent} at once · {day.peakTime}
                </span>
              </div>
            </div>

            {/* lanes */}
            {day.lanes.map((lane) => (
              <div key={lane.sessionId} className="mb-1 flex items-center gap-2">
                <div className="w-[110px] shrink-0 truncate text-[10.5px] text-muted-foreground" title={lane.title}>
                  {lane.title}
                </div>
                <div className="relative h-3.5 flex-1 rounded bg-white/[0.03]">
                  {lane.spans.map((s, i) => (
                    <span
                      key={i}
                      className="absolute top-0 h-3.5 rounded-sm bg-primary/70"
                      style={{ left: `${(s.startHour / 24) * 100}%`, width: `${Math.max(0.6, (s.durationHours / 24) * 100)}%` }}
                    />
                  ))}
                </div>
              </div>
            ))}
          </div>

          {/* cards grid */}
          <div className="grid grid-cols-2 gap-2.5">
            <StatCard label={`Leverage · ${day.label.split(",")[0]}`}>
              <div className="text-[24px] font-semibold tracking-tight">{report.leverage.ratio.toFixed(1)}×</div>
              <div className="text-[11px] text-muted-foreground">agent hours for every hour you were at your Mac</div>
              <div className="mt-2 flex h-1.5 overflow-hidden rounded-full">
                <span className="bg-primary" style={{ width: `${(report.leverage.hereHours / Math.max(1, report.leverage.agentHours)) * 100}%` }} />
                <span className="bg-status-dead" style={{ width: `${(report.leverage.awayHours / Math.max(1, report.leverage.agentHours)) * 100}%` }} />
              </div>
              <div className="mt-1.5 flex justify-between text-[10px] text-muted-foreground">
                <span>While you were here {report.leverage.hereHours}h</span>
                <span>away {report.leverage.awayHours}h</span>
              </div>
            </StatCard>

            <StatCard label="Waiting on you">
              <div className="text-[24px] font-semibold tracking-tight">{report.waiting.totalMinutes}m</div>
              <div className="text-[11px] text-muted-foreground">
                {report.waiting.count} times · longest {Math.round(report.waiting.longestS / 60)}m
              </div>
              <div className="mt-2 space-y-1">
                <Bucket label="Under a minute" n={report.waiting.under1m} max={40} color="bg-status-needsYou/80" />
                <Bucket label="1–5 minutes" n={report.waiting.m1To5} max={40} color="bg-status-needsYou/80" />
                <Bucket label="5–15 minutes" n={report.waiting.m5To15} max={40} color="bg-status-needsYou/80" />
                <Bucket label="15–60 minutes" n={report.waiting.m15To60} max={40} color="bg-status-needsYou/80" />
                <Bucket label="Over an hour" n={report.waiting.over60m} max={40} color="bg-status-needsYou/80" />
              </div>
            </StatCard>

            <StatCard label="Models">
              <div className="text-[18px] font-semibold tracking-tight">{report.topModel}</div>
              <div className="text-[11px] text-muted-foreground">did {report.topModelPct}% of the work</div>
              <div className="mt-2 space-y-1.5">
                {report.models.slice(0, 5).map((m) => (
                  <div key={m.model} className="flex items-center gap-2 text-[10.5px]">
                    <span className="w-[104px] truncate text-muted-foreground">{m.model}</span>
                    <span className="h-1 flex-1 rounded-full bg-white/8">
                      <span className="block h-1 rounded-full bg-primary" style={{ width: `${m.sharePct}%` }} />
                    </span>
                    <span className="w-10 text-right text-muted-foreground">{m.sharePct}%</span>
                  </div>
                ))}
              </div>
            </StatCard>

            <StatCard label="Subagents">
              <div className="text-[24px] font-semibold tracking-tight">{report.subagents.total.toLocaleString()}</div>
              <div className="text-[11px] text-muted-foreground">subagents sent off by your agents</div>
              <div className="mt-2 space-y-1.5">
                {[
                  ["Helper", report.subagents.helper],
                  ["Explore", report.subagents.explore],
                  ["General purpose", report.subagents.general],
                ].map(([label, n]) => (
                  <div key={String(label)} className="flex items-center gap-2 text-[10.5px]">
                    <span className="w-[92px] text-muted-foreground">{label}</span>
                    <span className="h-1 flex-1 rounded-full bg-white/8">
                      <span className="block h-1 rounded-full bg-primary/70" style={{ width: `${Math.min(100, (Number(n) / Math.max(1, report.subagents.total)) * 100)}%` }} />
                    </span>
                    <span className="w-8 text-right text-muted-foreground">{Number(n)}</span>
                  </div>
                ))}
              </div>
            </StatCard>

            <StatCard label="That's like…">
              {[report.thatsLike.hoursLine, report.thatsLike.tokensLine, report.thatsLike.costLine].map((line) => (
                <div key={line} className="mb-1.5 flex gap-2 text-[11.5px] text-muted-foreground">
                  <span className="mt-1 h-3 w-0.5 shrink-0 rounded bg-primary" />
                  <span>{line}</span>
                </div>
              ))}
            </StatCard>

            <StatCard label="Records · last 12 months">
              <div className="grid grid-cols-3 gap-2">
                <Record t="Longest turn" v={report.records.longestTurnLabel} s={report.records.longestTurnSub} />
                <Record t="Most at once" v={`${report.records.mostAtOnce} agents`} s={report.records.mostAtOnceSub} />
                <Record t="Busiest day" v={`${report.records.busiestDayTurns} turns`} s={report.records.busiestDaySub} />
                <Record t="Best streak" v={`${report.records.bestStreakDays} days`} s={report.records.bestStreakSub} />
                <Record t="Night owl" v={`${report.records.nightOwlTurns} turns`} s={report.records.nightOwlSub} />
                <Record t="Your rhythm" v={report.records.rhythmLabel} s={report.records.rhythmSub} />
              </div>
            </StatCard>
          </div>
        </>
      )}
    </div>
  );
}

function StatCard({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="rounded-xl border border-border/70 bg-card/50 p-3">
      <div className="mb-1.5 text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground/70">
        {label}
      </div>
      {children}
    </div>
  );
}

function Bucket({ label, n, max, color }: { label: string; n: number; max: number; color: string }) {
  return (
    <div className="flex items-center gap-2 text-[10.5px]">
      <span className="w-[104px] text-muted-foreground">{label}</span>
      <span className="h-1 flex-1 rounded-full bg-white/8">
        <span className={cn("block h-1 rounded-full", color)} style={{ width: `${Math.min(100, (n / max) * 100)}%` }} />
      </span>
      <span className="w-6 text-right text-muted-foreground">{n}</span>
    </div>
  );
}

function Record({ t, v, s }: { t: string; v: string; s: string }) {
  return (
    <div>
      <div className="text-[9.5px] uppercase tracking-wider text-muted-foreground/60">{t}</div>
      <div className="text-[13px] font-semibold tracking-tight">{v}</div>
      <div className="line-clamp-2 text-[9.5px] text-muted-foreground">{s}</div>
    </div>
  );
}
