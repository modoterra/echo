import { useEffect } from "react";
import { RiGithubFill } from "@remixicon/react";
import { GradientBackground } from "./components/gradient-background";
import { applyRandomization } from "./lib/randomize-bg";

export function HomePage() {
  useEffect(() => {
    applyRandomization();
  }, []);

  return (
    <main className="hero flex min-h-screen bg-white px-6 pb-20 pt-32 text-slate-950 sm:pt-36">
      <GradientBackground />
      <section className="mx-auto flex w-full max-w-5xl flex-col justify-center">
        <h1 className="text-center text-[clamp(1rem,4vw,2rem)] font-semibold tracking-normal text-slate-950">
          Echo Programming Language
        </h1>
        <p className="mx-auto text-balance mt-1 max-w-2xl text-center text-lg leading-8 text-slate-600 sm:text-xl">
          Echo is an all-in-one runtime for turning familiar code into fast
          native software.
        </p>
        <pre
          aria-label="Echo language example"
          className="mt-12 max-w-full overflow-hidden rounded-lg border border-transparent bg-white/0 px-5 py-5 text-left shadow-none backdrop-blur-none transition-all duration-300 ease-out hover:border-slate-200/70 hover:bg-white/60 hover:shadow-sm hover:backdrop-blur-sm focus-within:border-slate-200/70 focus-within:bg-white/60 focus-within:shadow-sm focus-within:backdrop-blur-sm sm:px-7 sm:py-6"
        >
          <code className="block whitespace-pre font-mono text-[clamp(0.8rem,1.8vw,1.125rem)] font-semibold leading-relaxed text-slate-950">
            <span className="text-sky-700">type</span>{" "}
            <span className="text-violet-700">App</span>{" "}
            <span className="text-slate-500">=</span>{" "}
            <span className="text-slate-500">{"{"}</span>
            {"\n"}
            {"    "}
            <span className="text-rose-700">name</span>
            <span className="text-slate-500">:</span>{" "}
            <span className="text-violet-700">string</span>
            {"\n"}
            {"    "}
            <span className="text-rose-700">runtime</span>
            <span className="text-slate-500">:</span>{" "}
            <span className="text-violet-700">string</span>
            {"\n"}
            <span className="text-slate-500">{"}"}</span>
            {"\n\n"}
            <span className="text-amber-700">$withEcho</span>{" "}
            <span className="text-slate-500">=</span>{" "}
            <span className="text-sky-700">defer</span>{" "}
            <span className="text-slate-500">{"{"}</span>
            {"\n"}
            {"    "}
            <span className="text-sky-700">let</span>{" "}
            <span className="text-amber-700">$app</span>{" "}
            <span className="text-slate-500">=</span>{" "}
            <span className="text-violet-700">App</span>{" "}
            <span className="text-slate-500">{"{"}</span>
            {"\n"}
            {"        "}
            <span className="text-rose-700">name</span>
            <span className="text-slate-500">:</span>{" "}
            <span className="text-emerald-700">"Echo"</span>
            {"\n"}
            {"        "}
            <span className="text-rose-700">runtime</span>
            <span className="text-slate-500">:</span>{" "}
            <span className="text-emerald-700">"native"</span>
            {"\n"}
            {"    "}
            <span className="text-slate-500">{"}"}</span>
            {"\n\n"}
            {"    "}
            <span className="text-sky-700">echo</span>{" "}
            <span className="text-amber-700">$app</span>
            <span className="text-slate-500">.</span>
            <span className="text-rose-700">name</span>{" "}
            <span className="text-slate-500">.</span>{" "}
            <span className="text-emerald-700">" runs "</span>{" "}
            <span className="text-slate-500">.</span>{" "}
            <span className="text-amber-700">$app</span>
            <span className="text-slate-500">.</span>
            <span className="text-rose-700">runtime</span>{" "}
            <span className="text-slate-500">.</span>{" "}
            <span className="text-emerald-700">"\n"</span>
            {"\n"}
            <span className="text-slate-500">{"}"}</span>
            {"\n\n"}
            <span className="text-sky-700">run</span>{" "}
            <span className="text-amber-700">$withEcho</span>
          </code>
        </pre>
        <div className="mt-16 max-w-2xl text-left">
          <h1 className="text-3xl font-semibold tracking-normal text-slate-950 sm:text-4xl">
            Familiar syntax. Native future.
          </h1>
          <p className="mt-5 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            Echo is a Rust powered PHP superset for modern server software. It
            keeps PHP familiar while adding native concurrency, parallel
            execution, sharper diagnostics, stronger tooling, and a path to
            compiled binaries with predictable performance gains. The xo mark
            nods to Echo by sound rather than spelling, folding /ˈɛkoʊ/ toward a
            compact /ɛk oʊ/ for a small command shaped for the terminal.
          </p>
          <a
            href="https://github.com/modoterra/echo"
            target="_blank"
            rel="noreferrer"
            className="mt-6 inline-flex items-center gap-2 text-sm font-medium text-slate-500 transition hover:text-slate-950"
          >
            <RiGithubFill aria-hidden="true" className="size-5" />
            GitHub
          </a>
        </div>
      </section>
    </main>
  );
}

export default HomePage;
