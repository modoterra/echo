import { useEffect } from "react";
import { RiGithubFill } from "@remixicon/react";
import { Logo } from "./components/logo";
import { GradientBackground } from "./components/gradient-background";
import { applyRandomization } from "./lib/randomize-bg";

function App() {
  // Apply randomization once on mount
  useEffect(() => {
    applyRandomization();
  }, []);

  return (
    <main className="hero flex min-h-screen items-center justify-center bg-white p-6 text-slate-950">
      <GradientBackground />
      <section className="hero-content relative h-[500px] w-full max-w-[624px] sm:h-[520px]">
        <Logo />
        <div className="absolute left-0 top-[58%] w-[69%] text-left">
          <p className="text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            Echo is a Rust powered PHP superset for modern server software. It keeps PHP familiar
            while adding native concurrency, parallel execution, sharper diagnostics, stronger
            tooling, and a path to compiled binaries with predictable performance gains. The xo mark
            nods to Echo by sound rather than spelling, folding /ˈɛkoʊ/ toward a compact /ɛk oʊ/ for
            a small command shaped for the terminal.
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
          <p className="mt-4 text-sm text-slate-400">© 2026 Modoterra Corporation</p>
        </div>
      </section>
    </main>
  );
}

export default App;
