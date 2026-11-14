import { cn } from "@/lib/utils";
import type { Status } from "@/lib/types";

const FILL: Record<Status, string> = {
  working: "bg-status-working",
  needsYou: "bg-status-needsYou animate-pulse-ring",
  done: "bg-status-done",
  idle: "bg-status-idle/60",
  dead: "bg-status-dead",
  unknown: "bg-transparent border-2 border-dashed border-status-unknown",
};

export function StatusDot({
  status,
  brand,
  size = "md",
  className,
}: {
  status: Status;
  brand: string;
  size?: "sm" | "md" | "lg";
  className?: string;
}) {
  const dim = size === "sm" ? "h-2 w-2" : size === "lg" ? "h-4 w-4" : "h-3 w-3";
  return (
    <span
      aria-label={status}
      className={cn("status-dot shrink-0", dim, brand, FILL[status], className)}
    />
  );
}
