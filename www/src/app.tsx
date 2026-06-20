import { useEffect, useState } from "react";
import { RiGithubFill } from "@remixicon/react";
import { motion, useReducedMotion } from "motion/react";
import { Logo } from "./components/logo";
import { GradientBackground } from "./components/gradient-background";
import { applyRandomization } from "./lib/randomize-bg";

export function HomePage() {
  const [isLoaded, setIsLoaded] = useState(false);
  const prefersReducedMotion = useReducedMotion();

  // Apply randomization once on mount
  useEffect(() => {
    applyRandomization();
  }, []);

  useEffect(() => {
    let isCancelled = false;
    let removeLoadListener = () => {};

    const waitForLoad =
      document.readyState === "complete"
        ? Promise.resolve()
        : new Promise<void>((resolve) => {
            const handleLoad = () => resolve();
            window.addEventListener("load", handleLoad, { once: true });
            removeLoadListener = () => window.removeEventListener("load", handleLoad);
          });

    const waitForLogoFont =
      document.fonts?.load("700 300px 'Space Grotesk'").then(() => undefined) ?? Promise.resolve();

    Promise.all([waitForLoad, waitForLogoFont]).then(() => {
      if (!isCancelled) {
        setIsLoaded(true);
      }
    });

    return () => {
      isCancelled = true;
      removeLoadListener();
    };
  }, []);

  return (
    <main className="hero flex min-h-screen items-center justify-center bg-white p-6 text-slate-950">
      <GradientBackground />
      <section className="hero-content relative h-[500px] w-full max-w-[624px] sm:h-[520px]">
        <motion.div
          initial={false}
          animate={{ opacity: isLoaded ? 1 : 0 }}
          transition={{ duration: prefersReducedMotion ? 0 : 0.45, ease: "easeOut" }}
        >
          <Logo />
        </motion.div>
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

export default HomePage;
