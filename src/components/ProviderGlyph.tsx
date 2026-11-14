import { Asterisk, Braces, Bot, Hexagon, MousePointer2, Sparkle } from "lucide-react";
import { cn } from "@/lib/utils";

const GLYPH: Record<string, typeof Asterisk> = {
  "claude-code": Asterisk,
  codex: Hexagon,
  opencode: Braces,
  cursor: MousePointer2,
  "gemini-cli": Sparkle,
  "copilot-cli": Bot,
};

/** The vendor mark. Small, muted, coloured — never louder than the status. */
export function ProviderGlyph({ provider, className }: { provider: string; className?: string }) {
  const Icon = GLYPH[provider] ?? Bot;
  return (
    <Icon
      className={cn("h-3.5 w-3.5 shrink-0", className)}
      style={{ color: `hsl(var(--brand))` }}
      strokeWidth={2.2}
    />
  );
}
