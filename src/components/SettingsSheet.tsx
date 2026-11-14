import { Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export interface Prefs {
  theme: "dark" | "light";
  accent: string;
  radius: number;
  glass: boolean;
}

const ACCENTS = ["211 100% 52%", "36 100% 52%", "135 71% 50%", "262 72% 62%", "347 100% 61%", "190 90% 55%"];

export function SettingsSheet({
  prefs,
  onChange,
  open,
}: {
  prefs: Prefs;
  onChange: (p: Prefs) => void;
  open: boolean;
}) {
  if (!open) return null;
  return (
    <aside className="glass w-[268px] shrink-0 border-l p-4">
      <h2 className="mb-3 text-[12px] font-semibold">Appearance</h2>

      <Field label="Theme">
        <div className="flex gap-1.5">
          <Button
            size="sm"
            variant={prefs.theme === "dark" ? "default" : "outline"}
            onClick={() => onChange({ ...prefs, theme: "dark" })}
          >
            <Moon className="h-3 w-3" /> Dark
          </Button>
          <Button
            size="sm"
            variant={prefs.theme === "light" ? "default" : "outline"}
            onClick={() => onChange({ ...prefs, theme: "light" })}
          >
            <Sun className="h-3 w-3" /> Light
          </Button>
        </div>
      </Field>

      <Field label="Accent">
        <div className="flex gap-2">
          {ACCENTS.map((a) => (
            <button
              key={a}
              onClick={() => onChange({ ...prefs, accent: a })}
              aria-label={`accent ${a}`}
              className={cn(
                "h-[18px] w-[18px] rounded-full border-2 transition-transform hover:scale-110",
                prefs.accent === a ? "border-foreground" : "border-transparent"
              )}
              style={{ background: `hsl(${a})` }}
            />
          ))}
        </div>
      </Field>

      <Field label={`Corner radius · ${prefs.radius}px`}>
        <input
          type="range"
          min={2}
          max={18}
          value={prefs.radius}
          onChange={(e) => onChange({ ...prefs, radius: Number(e.target.value) })}
          className="w-full accent-[hsl(var(--primary))]"
        />
      </Field>

      <Field label="Glass">
        <button
          onClick={() => onChange({ ...prefs, glass: !prefs.glass })}
          className={cn(
            "h-6 w-11 rounded-full p-0.5 transition-colors",
            prefs.glass ? "bg-primary" : "bg-muted"
          )}
          aria-label="toggle glass"
        >
          <span
            className={cn(
              "block h-5 w-5 rounded-full bg-white transition-transform",
              prefs.glass && "translate-x-5"
            )}
          />
        </button>
      </Field>
    </aside>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="border-t border-border/60 py-3 first:border-t-0 first:pt-0">
      <div className="mb-2 text-[11px] text-muted-foreground">{label}</div>
      {children}
    </div>
  );
}
