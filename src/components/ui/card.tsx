import * as React from "react";
import { cn } from "@/lib/utils";

export const Card = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={cn(
        "rounded-lg border border-border bg-card/70 backdrop-blur-xl transition-colors",
        className
      )}
      {...props}
    />
  )
);
Card.displayName = "Card";
