import { useEffect, useState } from "react";
import {
  Check, CreditCard, FolderGit2, KeyRound, Palette, Smartphone, Sparkles, X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { AgentCatalogItem, DiagnosticsReport, HostAppItem, IphoneStatus } from "@/lib/types";
import * as api from "@/lib/bridge";
import { StatusDot } from "./StatusDot";

type Tab = "general" | "inbox" | "appearance" | "agents" | "apps" | "iphone" | "license";

const TABS: { id: Tab; label: string; icon: typeof Palette }[] = [
  { id: "general", label: "General", icon: Sparkles },
  { id: "inbox", label: "Inbox", icon: FolderGit2 },
  { id: "appearance", label: "Appearance", icon: Palette },
  { id: "agents", label: "Agents", icon: Sparkles },
  { id: "apps", label: "Apps", icon: FolderGit2 },
  { id: "iphone", label: "iPhone", icon: Smartphone },
  { id: "license", label: "License", icon: CreditCard },
];

export interface Prefs {
  theme: "dark" | "light" | "system";
  accent: string;
  radius: number;
  rows: "tiles" | "list" | "compact";
  stripPosition: "right" | "left" | "dock" | "notch" | "menubar";
  showNames: boolean;
  providerIcons: boolean;
  foldAfter: string;
  popup: string;
  notify: string;
  sound: boolean;
  bounce: boolean;
  flash: boolean;
  collisions: boolean;
  comeback: string;
}

export const DEFAULT_PREFS: Prefs = {
  theme: "dark", accent: "211 100% 52%", radius: 12, rows: "tiles",
  stripPosition: "right", showNames: false, providerIcons: false, foldAfter: "15 minutes",
  popup: "Off", notify: "Turns over 1 minute", sound: true, bounce: false, flash: true,
  collisions: true, comeback: "After 15 minutes away",
};

export function SettingsView({
  prefs, setPrefs, agents, hosts, diagnostics, iphone,
}: {
  prefs: Prefs;
  setPrefs: (p: Prefs) => void;
  agents: AgentCatalogItem[];
  hosts: HostAppItem[];
  diagnostics: DiagnosticsReport | null;
  iphone: IphoneStatus | null;
}) {
  const [tab, setTab] = useState<Tab>("general");
  const [selectedAgent, setSelectedAgent] = useState("claude-code");
  const [selectedHost, setSelectedHost] = useState("cmux");
  const [hookOn, setHookOn] = useState(diagnostics?.claudeHookInstalled ?? false);
  const [mcpOn, setMcpOn] = useState(diagnostics?.claudeMcpRegistered ?? false);
  const [phone, setPhone] = useState<IphoneStatus | null>(iphone);

  useEffect(() => {
    api.iphoneStatus().then(setPhone).catch(() => {});
  }, []);

  const agent = agents.find((a) => a.id === selectedAgent) ?? agents[0];
  const host = hosts.find((h) => h.id === selectedHost) ?? hosts[0];

  return (
    <div className="flex h-full flex-col">
      {/* tab bar */}
      <div className="flex shrink-0 items-center gap-1 border-b border-border/70 px-4 py-1.5">
        {TABS.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setTab(id)}
            className={cn(
              "flex flex-col items-center gap-0.5 rounded-md px-3 py-1.5 text-[10.5px] transition-colors",
              tab === id ? "bg-secondary/80 text-foreground" : "text-muted-foreground hover:text-foreground"
            )}
          >
            <Icon className="h-3.5 w-3.5" />
            {label}
          </button>
        ))}
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto p-4">
        {tab === "general" && (
          <div className="mx-auto max-w-[560px] space-y-4">
            <Section title="Strip">
              <Row label="Position">
                <Select
                  value={prefs.stripPosition}
                  onChange={(v) => {
                    setPrefs({ ...prefs, stripPosition: v as Prefs["stripPosition"] });
                    api.setStripPosition(v);
                  }}
                  options={[
                    ["right", "Right edge"],
                    ["left", "Left edge"],
                    ["dock", "By the Dock"],
                    ["notch", "Around the notch"],
                    ["menubar", "Only in Menu Bar"],
                  ]}
                />
              </Row>
              <Row label="Always show agent names">
                <Switch on={prefs.showNames} onChange={(v) => setPrefs({ ...prefs, showNames: v })} />
              </Row>
              <Row label="Clicking an agent that needs you">
                <Select value="inbox" onChange={() => {}} options={[["inbox", "Opens its card in the Inbox"], ["agent", "Goes to the agent"]]} />
              </Row>
              <Row label="Fold idle agents after">
                <Select
                  value={prefs.foldAfter}
                  onChange={(v) => setPrefs({ ...prefs, foldAfter: v })}
                  options={[["5 minutes", "5 minutes"], ["15 minutes", "15 minutes"], ["30 minutes", "30 minutes"], ["1 hour", "1 hour"], ["Never", "Never"]]}
                />
              </Row>
            </Section>

            <Section title="Shortcuts">
              <Row label="Ask with ⌥⌥"><Badge ok>✓ Allowed</Badge></Row>
              <Row label="Inbox"><kbd className="rounded bg-white/10 px-1.5 py-0.5 font-mono text-[10px]">⌃⌥I</kbd></Row>
              <Row label="Hover card"><kbd className="rounded bg-white/10 px-1.5 py-0.5 font-mono text-[10px]">⌃⌥H</kbd></Row>
            </Section>

            <Section title="AgentWatch">
              <Row label="Launch at login"><Switch on={false} onChange={() => {}} /></Row>
              <Row label="Check for updates automatically"><Switch on onChange={() => {}} /></Row>
              <Row label="Clone Code hook (byte-identical backup)">
                <div className="flex items-center gap-2">
                  <Badge ok={hookOn}>{hookOn ? "Installed" : "Off"}</Badge>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={async () => {
                      if (hookOn) { await api.hooksUninstall(); setHookOn(false); }
                      else { await api.hooksInstall(); setHookOn(true); }
                    }}
                  >
                    {hookOn ? "Uninstall" : "Install hook"}
                  </Button>
                </div>
              </Row>
              <Row label="MCP server (~/.claude.json)">
                <div className="flex items-center gap-2">
                  <Badge ok={mcpOn}>{mcpOn ? "Registered" : "Off"}</Badge>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={async () => { await api.mcpToggle(!mcpOn); setMcpOn(!mcpOn); }}
                  >
                    {mcpOn ? "Remove" : "Register"}
                  </Button>
                </div>
              </Row>
            </Section>
          </div>
        )}

        {tab === "inbox" && (
          <div className="mx-auto max-w-[560px] space-y-4">
            <Section title="Decision pop-up">
              <Row label="Pop up questions">
                <Select
                  value={prefs.popup}
                  onChange={(v) => setPrefs({ ...prefs, popup: v })}
                  options={[["Off", "Off"], ["Center of the screen", "Center of the screen"], ["Top of the screen", "Top of the screen"]]}
                />
              </Row>
              <Row label="Keyboard goes to it"><Select value="second" onChange={() => {}} options={[["second", "A second after it appears"], ["click", "After ⌃⌥I or a click"]]} /></Row>
            </Section>
            <Section title="When an agent finishes">
              <Row label="Notify">
                <Select
                  value={prefs.notify}
                  onChange={(v) => setPrefs({ ...prefs, notify: v })}
                  options={[["Off", "Off"], ["After every turn", "After every turn"], ["Turns over 1 minute", "Turns over 1 minute"], ["Turns over 5 minutes", "Turns over 5 minutes"]]}
                />
              </Row>
              <Row label="Flash where the dots are"><Switch on={prefs.flash} onChange={(v) => setPrefs({ ...prefs, flash: v })} /></Row>
              <Row label="Bounce the Dock icon"><Switch on={prefs.bounce} onChange={(v) => setPrefs({ ...prefs, bounce: v })} /></Row>
              <Row label="Play a sound"><Switch on={prefs.sound} onChange={(v) => setPrefs({ ...prefs, sound: v })} /></Row>
            </Section>
            <Section title="Also tell you">
              <Row label="Two agents change the same file"><Switch on={prefs.collisions} onChange={(v) => setPrefs({ ...prefs, collisions: v })} /></Row>
              <Row label="Summary when you come back">
                <Select
                  value={prefs.comeback}
                  onChange={(v) => setPrefs({ ...prefs, comeback: v })}
                  options={[["After 10 minutes away", "After 10 minutes away"], ["After 15 minutes away", "After 15 minutes away"], ["After 30 minutes away", "After 30 minutes away"]]}
                />
              </Row>
            </Section>
          </div>
        )}

        {tab === "appearance" && (
          <div className="grid grid-cols-[1fr_320px] gap-4">
            <div className="space-y-4">
              <Section title="Look">
                <Row label="Appearance">
                  <Select
                    value={prefs.theme}
                    onChange={(v) => setPrefs({ ...prefs, theme: v as Prefs["theme"] })}
                    options={[["system", "System"], ["dark", "Dark"], ["light", "Light"]]}
                  />
                </Row>
                <Row label="Rows">
                  <Select
                    value={prefs.rows}
                    onChange={(v) => setPrefs({ ...prefs, rows: v as Prefs["rows"] })}
                    options={[["tiles", "Tiles"], ["list", "List"], ["compact", "Compact"]]}
                  />
                </Row>
                <Row label="Provider icons"><Switch on={prefs.providerIcons} onChange={(v) => setPrefs({ ...prefs, providerIcons: v })} /></Row>
                <Row label={`Corner radius · ${prefs.radius}px`}>
                  <input
                    type="range" min={4} max={20} value={prefs.radius}
                    onChange={(e) => setPrefs({ ...prefs, radius: Number(e.target.value) })}
                    className="w-32 accent-[hsl(var(--primary))]"
                  />
                </Row>
              </Section>
              <Section title="Accent">
                <div className="flex gap-2 px-1 py-1.5">
                  {["211 100% 52%", "36 100% 52%", "135 71% 50%", "262 72% 62%", "347 100% 61%", "190 90% 55%"].map((a) => (
                    <button
                      key={a}
                      onClick={() => setPrefs({ ...prefs, accent: a })}
                      className={cn("h-5 w-5 rounded-full border-2", prefs.accent === a ? "border-foreground" : "border-transparent")}
                      style={{ background: `hsl(${a})` }}
                    />
                  ))}
                </div>
              </Section>
            </div>

            {/* live preview */}
            <div className="rounded-xl border border-border/70 bg-card/40 p-2.5">
              <div className="mb-2 flex items-center justify-between px-1">
                <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/70">Live preview</span>
                <span className="text-[10px] text-muted-foreground">Mark all as seen</span>
              </div>
              {[
                { t: "Pricing page layout", s: "needsYou" as const, r: "shop-web", b: "feature/pricing", d: "1m" },
                { t: "Add Stripe checkout", s: "needsYou" as const, r: "shop-api", b: "main", d: "3m" },
                { t: "Fix the flaky login test", s: "working" as const, r: "shop-api", b: "main", d: "23:46" },
                { t: "Research pricing pages", s: "working" as const, r: "shop-web", b: "main", d: "23:46" },
                { t: "Tidy the README", s: "done" as const, r: "notes-ios", b: "readme", d: "4m" },
                { t: "Release notes for 2.4", s: "idle" as const, r: "notes-ios", b: "release/2.4", d: "9m" },
              ].map((row) => (
                <div
                  key={row.t}
                  className={cn(
                    "mb-1 flex items-center gap-2 rounded-md border px-2 py-1.5",
                    row.s === "needsYou" ? "border-status-needsYou/40 bg-status-needsYou/[0.09]" : "border-border/50 bg-black/20"
                  )}
                >
                  <div className="min-w-0 flex-1">
                    <div className="truncate text-[10.5px] font-medium">{row.t}</div>
                    <div className="truncate text-[9.5px] text-muted-foreground">
                      {row.r} ⎇ {row.b}
                    </div>
                  </div>
                  <span className="font-mono text-[9px] text-muted-foreground">{row.d}</span>
                  <StatusDot status={row.s} brand="brand-claude" size="sm" />
                </div>
              ))}
            </div>
          </div>
        )}

        {tab === "agents" && (
          <div className="grid h-full grid-cols-[210px_1fr] gap-3">
            <div className="overflow-y-auto rounded-xl border border-border/70 bg-card/40 p-1.5">
              {["supported", "preview", "cmux"].map((tier) => {
                const list = agents.filter((a) => a.tier === tier);
                if (!list.length) return null;
                return (
                  <div key={tier} className="mb-2">
                    <div className="px-2 py-1 text-[9.5px] font-semibold uppercase tracking-wider text-muted-foreground/60">
                      {tier === "cmux" ? "In cmux and herdr" : tier}
                    </div>
                    {list.map((a) => (
                      <button
                        key={a.id}
                        onClick={() => setSelectedAgent(a.id)}
                        className={cn(
                          "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-[11.5px]",
                          selectedAgent === a.id ? "bg-secondary/80 text-foreground" : "text-muted-foreground hover:bg-secondary/40"
                        )}
                      >
                        <span className="w-4 text-center text-[10px] opacity-70">{a.short}</span>
                        <span className="flex-1 truncate">{a.name}</span>
                        <span className={cn("h-1.5 w-1.5 rounded-full", a.installed ? "bg-status-done" : "bg-white/15")} />
                      </button>
                    ))}
                  </div>
                );
              })}
            </div>

            {agent && (
              <div className="overflow-y-auto rounded-xl border border-border/70 bg-card/40 p-4">
                <div className="flex items-center gap-3">
                  <div className="grid h-10 w-10 place-items-center rounded-lg bg-white/8 text-[15px]">{agent.short}</div>
                  <div>
                    <div className="text-[15px] font-semibold tracking-tight">{agent.name}</div>
                    <div className={cn("mt-0.5 inline-flex rounded-full border px-2 py-px text-[10px]", agent.installed ? "border-status-done/40 text-status-done" : "border-border text-muted-foreground")}>
                      {agent.installed ? `Installed: ${agent.path}` : "Not installed"}
                    </div>
                  </div>
                </div>

                <p className="mt-3 text-[12px] leading-relaxed text-muted-foreground">{agent.description}</p>

                <div className="mt-3 space-y-1 rounded-lg border border-border/60 bg-black/20 p-2.5">
                  {[
                    ["See what it's doing", agent.canSee],
                    ["History, Activity and Tokens", agent.canHistory],
                    ["Answer questions from the Inbox", agent.canAnswer],
                    ["Type replies and ⌥⌥ messages", agent.canType],
                  ].map(([label, value]) => {
                    const ok = value === "yes" || value.startsWith("yes");
                    const no = value === "not yet" || value === "elsewhere";
                    return (
                      <div key={String(label)} className="flex items-center gap-2 text-[11.5px]">
                        {ok ? (
                          <Check className="h-3 w-3 text-status-done" />
                        ) : no ? (
                          <X className="h-3 w-3 text-muted-foreground/50" />
                        ) : (
                          <span className="grid h-3 w-3 place-items-center rounded-full bg-white/10 text-[8px]">–</span>
                        )}
                        <span className={no ? "text-muted-foreground/60" : "text-foreground"}>{label}</span>
                        <span className="ml-auto text-[10.5px] text-muted-foreground">{String(value)}</span>
                      </div>
                    );
                  })}
                </div>

                <div className="mt-3 flex items-center gap-2">
                  <span className="text-[11.5px] text-muted-foreground">Show {agent.name} sessions</span>
                  <Switch on={agent.installed} onChange={() => {}} />
                </div>
              </div>
            )}
          </div>
        )}

        {tab === "apps" && (
          <div className="grid h-full grid-cols-[210px_1fr] gap-3">
            <div className="overflow-y-auto rounded-xl border border-border/70 bg-card/40 p-1.5">
              {hosts.map((h) => (
                <button
                  key={h.id}
                  onClick={() => setSelectedHost(h.id)}
                  className={cn(
                    "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-[11.5px]",
                    selectedHost === h.id ? "bg-secondary/80 text-foreground" : "text-muted-foreground hover:bg-secondary/40"
                  )}
                >
                  <span className="flex-1 truncate">{h.name}</span>
                  <span className={cn("h-1.5 w-1.5 rounded-full", h.statusOk ? "bg-status-done" : "bg-white/15")} />
                </button>
              ))}
            </div>
            {host && (
              <div className="overflow-y-auto rounded-xl border border-border/70 bg-card/40 p-4">
                <div className="text-[15px] font-semibold tracking-tight">{host.name}</div>
                <div className={cn("mt-1 inline-flex rounded-full border px-2 py-px text-[10px]", host.statusOk ? "border-status-done/40 text-status-done" : "border-border text-muted-foreground")}>
                  {host.statusBadge}
                </div>
                <p className="mt-3 text-[12px] leading-relaxed text-muted-foreground">{host.summary}</p>
                <div className="mt-3 grid grid-cols-4 gap-2 text-[10px]">
                  {[["See", host.canSee], ["Answer", host.canAnswer !== "no"], ["Type", host.canType], ["Jump", host.canJump]].map(([l, ok]) => (
                    <div key={String(l)} className="rounded-md border border-border/60 bg-black/20 p-1.5 text-center">
                      <div className="text-muted-foreground/60">{String(l)}</div>
                      <div className={ok ? "text-status-done" : "text-muted-foreground/50"}>{ok ? "✓" : "–"}</div>
                    </div>
                  ))}
                </div>
                <ol className="mt-3 list-decimal space-y-1 pl-4 text-[11.5px] text-muted-foreground">
                  {host.setupSteps.map((s) => <li key={s}>{s}</li>)}
                </ol>
                <div className="mt-3 rounded-md bg-black/30 px-2.5 py-2 font-mono text-[10.5px] text-muted-foreground/80">
                  {host.diagnostic}
                </div>
              </div>
            )}
          </div>
        )}

        {tab === "iphone" && (
          <div className="mx-auto max-w-[520px] space-y-4">
            <Section title="Answer from your iPhone">
              <Row label="Enable LAN companion page">
                <Switch
                  on={phone?.enabled ?? false}
                  onChange={async (v) => setPhone(await api.iphoneToggle(v))}
                />
              </Row>
              {phone && (
                <>
                  <Row label="Local URL">
                    <code className="max-w-[280px] truncate rounded bg-black/30 px-2 py-1 font-mono text-[10.5px]">{phone.url}</code>
                  </Row>
                  <Row label="Rotate pairing token">
                    <Button size="sm" variant="outline" onClick={async () => setPhone(await api.iphoneRotate())}>
                      <KeyRound className="h-3 w-3" /> New link
                    </Button>
                  </Row>
                  <Row label="Relay (E2E encrypted)">
                    <input
                      placeholder="relay://… (optional)"
                      className="w-[200px] rounded border border-border/70 bg-black/25 px-2 py-1 text-[11px] outline-none"
                    />
                  </Row>
                </>
              )}
            </Section>
            <p className="px-1 text-[11px] leading-relaxed text-muted-foreground">
              The page is served from this Mac on port {phone?.port ?? 47811} and only devices with the
              link can connect. Nothing leaves your network.
            </p>
          </div>
        )}

        {tab === "license" && (
          <div className="mx-auto max-w-[520px] space-y-4">
            <Section title="Licence">
              <Row label="Plan"><Badge ok>Free · local &amp; private</Badge></Row>
              <Row label="Machines"><span className="text-[12px] text-muted-foreground">This Mac</span></Row>
              <Row label="Purchase (optional)">
                <Button size="sm" variant="outline"><CreditCard className="h-3 w-3" /> $19 one-time · 3 Macs</Button>
              </Row>
              <Row label="License key">
                <input placeholder="Paste key…" className="w-[180px] rounded border border-border/70 bg-black/25 px-2 py-1 text-[11px] outline-none" />
              </Row>
            </Section>
            <p className="px-1 text-[11px] text-muted-foreground">
              Every feature works without a licence while AgentWatch is in development.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section>
      <h3 className="mb-1.5 px-1 text-[12px] font-semibold">{title}</h3>
      <div className="overflow-hidden rounded-xl border border-border/70 bg-card/50">{children}</div>
    </section>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center gap-3 border-b border-border/40 px-3.5 py-2.5 last:border-b-0">
      <span className="text-[12px] text-foreground/90">{label}</span>
      <div className="ml-auto flex items-center gap-2">{children}</div>
    </div>
  );
}

function Select({ value, onChange, options }: { value: string; onChange: (v: string) => void; options: [string, string][] }) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="rounded-md border border-border/70 bg-black/30 px-2 py-1 text-[11.5px] text-foreground outline-none"
    >
      {options.map(([v, l]) => <option key={v} value={v}>{l}</option>)}
    </select>
  );
}

function Switch({ on, onChange }: { on: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!on)}
      className={cn("h-5 w-9 rounded-full p-0.5 transition-colors", on ? "bg-primary" : "bg-white/15")}
      aria-pressed={on}
    >
      <span className={cn("block h-4 w-4 rounded-full bg-white transition-transform", on && "translate-x-4")} />
    </button>
  );
}

function Badge({ ok, children }: { ok: boolean; children: React.ReactNode }) {
  return (
    <span className={cn("inline-flex items-center gap-1 rounded-full border px-2 py-px text-[10px]",
      ok ? "border-status-done/40 text-status-done" : "border-border text-muted-foreground")}>
      {children}
    </span>
  );
}
