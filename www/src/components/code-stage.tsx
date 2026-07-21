import { useId, useState, type KeyboardEvent } from "react";
import { Link } from "@tanstack/react-router";
import { EchoCode } from "./echo-code";

export type CodeStageDemo = {
  id: string;
  label: string;
  blurb: string;
  code: string;
  language?: "echo" | "shellscript";
  command?: string;
  output: string;
  docsHref: string;
  docsLabel: string;
};

type Props = {
  demos: CodeStageDemo[];
  title?: string;
  subtitle?: string;
};

/**
 * Tabbed “see it work” stage: code + shell output + docs link.
 * Keyboard: arrow keys move between tabs when focus is in the tablist.
 */
export function CodeStage({
  demos,
  title = "See it work",
  subtitle = "Four small programs that show the surface. Static demos — run them after Install.",
}: Props) {
  const baseId = useId();
  const [active, setActive] = useState(0);
  const demo = demos[active] ?? demos[0];

  if (!demo) {
    return null;
  }

  function onTabKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    if (
      event.key !== "ArrowRight" &&
      event.key !== "ArrowLeft" &&
      event.key !== "Home" &&
      event.key !== "End"
    ) {
      return;
    }
    event.preventDefault();
    let next = active;
    if (event.key === "ArrowRight") {
      next = (active + 1) % demos.length;
    } else if (event.key === "ArrowLeft") {
      next = (active - 1 + demos.length) % demos.length;
    } else if (event.key === "Home") {
      next = 0;
    } else if (event.key === "End") {
      next = demos.length - 1;
    }
    setActive(next);
    const btn = document.getElementById(`${baseId}-tab-${next}`);
    btn?.focus();
  }

  return (
    <section className="mt-28 sm:mt-36" aria-labelledby={`${baseId}-heading`}>
      <h2
        id={`${baseId}-heading`}
        className="max-w-4xl text-balance font-display text-4xl font-bold leading-[1.02] tracking-[-0.045em] text-slate-950 sm:text-5xl"
      >
        {title}
      </h2>
      <p className="mt-6 max-w-3xl text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
        {subtitle}
      </p>

      <div className="mt-10 overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-xl shadow-slate-950/6">
        <div
          role="tablist"
          aria-label="Language demos"
          className="scrollbar-nice flex flex-nowrap gap-1 overflow-x-auto border-b border-slate-200 bg-slate-50/80 px-3 py-3 sm:px-5"
          onKeyDown={onTabKeyDown}
        >
          {demos.map((item, index) => {
            const selected = index === active;
            return (
              <button
                key={item.id}
                type="button"
                role="tab"
                id={`${baseId}-tab-${index}`}
                aria-selected={selected}
                aria-controls={`${baseId}-panel`}
                tabIndex={selected ? 0 : -1}
                className={
                  selected
                    ? "rounded-lg bg-white px-3.5 py-2 text-sm font-semibold text-violet-700 shadow-sm ring-1 ring-violet-200"
                    : "rounded-lg px-3.5 py-2 text-sm font-semibold text-slate-500 transition hover:bg-white hover:text-slate-950"
                }
                onClick={() => setActive(index)}
              >
                {item.label}
              </button>
            );
          })}
        </div>

        <div
          role="tabpanel"
          id={`${baseId}-panel`}
          aria-labelledby={`${baseId}-tab-${active}`}
          className="grid gap-0 lg:grid-cols-[minmax(0,1.2fr)_minmax(0,0.8fr)]"
        >
          <div className="border-b border-slate-200 bg-slate-50/50 p-4 sm:p-8 lg:border-b-0 lg:border-r">
            <EchoCode
              code={demo.code}
              language={demo.language ?? "echo"}
              variant="inline-block"
              className="mt-0 min-h-80 overflow-x-auto rounded-2xl border border-slate-200 bg-white shadow-sm"
            />
          </div>
          <div className="flex flex-col justify-between gap-8 p-6 sm:p-8">
            <div>
              <p className="font-display text-2xl font-bold tracking-tight text-slate-950">
                {demo.label}
              </p>
              <p className="mt-3 text-sm leading-6 text-slate-600 sm:text-base sm:leading-7">
                {demo.blurb}
              </p>
              <div className="mt-6 overflow-hidden rounded-xl border border-slate-800 bg-slate-950 px-4 py-4 font-mono text-sm leading-7 text-slate-100 shadow-lg shadow-slate-950/10">
                <p className="text-xs font-semibold tracking-wide text-slate-400">Output</p>
                {demo.command ? (
                  <p className="mt-2 text-slate-400">
                    <span className="text-slate-500">$ </span>
                    {demo.command}
                  </p>
                ) : null}
                <pre className="mt-1 whitespace-pre-wrap text-emerald-300">{demo.output}</pre>
              </div>
            </div>
            <Link
              className="text-sm font-semibold text-violet-700 transition hover:text-violet-900"
              to={demo.docsHref as "/"}
            >
              {demo.docsLabel} →
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}
