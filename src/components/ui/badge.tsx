import * as React from "react";
import { cn } from "@/lib/utils";

export function Badge({
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full px-2 py-[2px] text-[10px] font-semibold tracking-wide",
        className
      )}
      {...props}
    >
      {children}
    </span>
  );
}
