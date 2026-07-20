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
    <section className="mt-24 sm:mt-28" aria-labelledby={`${baseId}-heading`}>
      <h2
        id={`${baseId}-heading`}
        className="text-2xl font-semibold tracking-normal text-slate-950 sm:text-3xl"
      >
        {title}
      </h2>
      <p className="mt-4 max-w-2xl text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
        {subtitle}
      </p>

      <div className="mt-8 overflow-hidden rounded-xl border border-slate-200 bg-white/90 shadow-sm">
        <div
          role="tablist"
          aria-label="Language demos"
          className="flex flex-wrap gap-1 border-b border-slate-200 bg-slate-50 px-2 py-2 sm:px-3"
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
                    ? "rounded-md bg-white px-3 py-2 text-sm font-semibold text-slate-950 shadow-sm ring-1 ring-slate-200"
                    : "rounded-md px-3 py-2 text-sm font-semibold text-slate-500 transition hover:bg-white/70 hover:text-slate-950"
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
          className="grid gap-0 lg:grid-cols-[minmax(0,1.15fr)_minmax(0,0.85fr)]"
        >
          <div className="border-b border-slate-200 p-4 sm:p-6 lg:border-b-0 lg:border-r">
            <EchoCode
              code={demo.code}
              language={demo.language ?? "echo"}
              variant="inline-block"
              className="mt-0 overflow-x-auto rounded-lg border border-slate-200 bg-slate-50"
            />
          </div>
          <div className="flex flex-col justify-between gap-6 p-4 sm:p-6">
            <div>
              <p className="text-base font-semibold text-slate-950 sm:text-lg">{demo.label}</p>
              <p className="mt-3 text-sm leading-6 text-slate-600 sm:text-base sm:leading-7">
                {demo.blurb}
              </p>
              <div className="mt-5 overflow-hidden rounded-lg border border-slate-800 bg-slate-950 px-4 py-4 font-mono text-sm leading-7 text-slate-100">
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
              className="text-sm font-semibold text-slate-800 underline-offset-4 hover:underline"
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
