import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";

type Variant = "primary" | "secondary" | "ghost";

type Props = {
  to: string;
  children: ReactNode;
  variant?: Variant;
  className?: string;
};

const VARIANT: Record<Variant, string> = {
  primary:
    "inline-flex items-center justify-center rounded-md bg-slate-950 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-slate-800 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-400 focus-visible:ring-offset-2",
  secondary:
    "inline-flex items-center justify-center rounded-md border border-slate-300 bg-white px-4 py-2.5 text-sm font-semibold text-slate-800 transition hover:border-slate-400 hover:text-slate-950 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-300 focus-visible:ring-offset-2",
  ghost:
    "inline-flex items-center justify-center rounded-md px-3 py-2 text-sm font-semibold text-slate-600 transition hover:text-slate-950 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-300",
};

const COMPACT =
  "inline-flex items-center justify-center rounded-md bg-slate-950 px-3 py-2 text-xs font-semibold text-white transition hover:bg-slate-800 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-400 focus-visible:ring-offset-2 sm:text-sm";

/**
 * Primary navigation / marketing CTAs. Paths outside the typed route union
 * use the same cast pattern as the rest of the site.
 */
export function CtaLink({
  to,
  children,
  variant = "primary",
  className,
  compact = false,
}: Props & { compact?: boolean }) {
  const base = compact && variant === "primary" ? COMPACT : VARIANT[variant];
  const classes = className ? `${base} ${className}` : base;
  return (
    <Link className={classes} to={to as "/"}>
      {children}
    </Link>
  );
}
