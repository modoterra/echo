import { useEffect } from "react";
import { RiDiscordFill, RiGithubFill, RiTwitterXFill } from "@remixicon/react";
import { Link } from "@tanstack/react-router";
import { EchoCode } from "./components/echo-code";
import { GradientBackground } from "./components/gradient-background";
import { applyRandomization } from "./lib/randomize-bg";

const HERO_ECHO = `/ std/io

$ xs = [1, 2, 3]
~ sum = 0
* x : xs {
    ~ sum = sum + x
}
io.print("sum={sum}")
`;

const GET_STARTED_SHELL = `cargo build -p xo
./target/debug/xo run examples/misc/hello.echo
`;

export function HomePage() {
  useEffect(() => {
    applyRandomization();
  }, []);

  return (
    <main className="hero flex min-h-screen bg-white px-6 pb-20 pt-32 text-slate-950 sm:pt-36">
      <GradientBackground />
      <section className="mx-auto flex w-full max-w-5xl flex-col justify-center">
        <h1 className="text-center text-[clamp(1rem,4vw,2rem)] font-semibold tracking-normal text-slate-950">
          Echo
        </h1>
        <p className="mx-auto mt-1 max-w-2xl text-balance text-center text-lg leading-8 text-slate-600 sm:text-xl">
          A compiled language for writing clear programs and shipping native binaries.
        </p>
        <EchoCode
          aria-label="Echo language example"
          code={HERO_ECHO}
          language="echo"
          variant="hero"
        />

        <div className="mt-20 grid max-w-3xl gap-14 text-left sm:mt-24">
          <section>
            <h2 className="text-2xl font-semibold tracking-normal text-slate-950 sm:text-3xl">
              No keywords
            </h2>
            <p className="mt-4 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
              A statement starts with a leader:{" "}
              <span className="font-mono font-semibold text-slate-800">$</span> binds,{" "}
              <span className="font-mono font-semibold text-slate-800">~</span> mutates,{" "}
              <span className="font-mono font-semibold text-slate-800">?</span> branches,{" "}
              <span className="font-mono font-semibold text-slate-800">*</span> loops,{" "}
              <span className="font-mono font-semibold text-slate-800">^</span> returns. The rest is
              ordinary expressions — nothing to memorize beyond a short glyph set.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-semibold tracking-normal text-slate-950 sm:text-3xl">
              Errors are values
            </h2>
            <p className="mt-4 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
              <span className="font-mono font-semibold text-slate-800">!</span> returns an error
              from the function; it does not abort the process. You match the result, or the
              program does not compile. Same idea for optionals.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-semibold tracking-normal text-slate-950 sm:text-3xl">
              Small surface, native output
            </h2>
            <p className="mt-4 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
              Imports stay module-scoped. I/O comes from{" "}
              <span className="font-mono font-semibold text-slate-800">std</span>, not free globals.{" "}
              <span className="font-mono font-semibold text-slate-800">xo run</span> and{" "}
              <span className="font-mono font-semibold text-slate-800">xo build</span> are the
              everyday loop.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-semibold tracking-normal text-slate-950 sm:text-3xl">
              Get started
            </h2>
            <p className="mt-4 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
              Clone the repo, build{" "}
              <span className="font-mono font-semibold text-slate-800">xo</span>, run an example.
              Read the{" "}
              <Link
                className="font-semibold text-slate-800 underline-offset-4 hover:underline"
                to={"/docs" as "/"}
              >
                Docs
              </Link>
              , the{" "}
              <Link
                className="font-semibold text-slate-800 underline-offset-4 hover:underline"
                to={"/book" as "/"}
              >
                Book
              </Link>
              , or{" "}
              <Link
                className="font-semibold text-slate-800 underline-offset-4 hover:underline"
                to={"/e26" as "/"}
              >
                e26
              </Link>
              .
            </p>
            <EchoCode code={GET_STARTED_SHELL} language="shellscript" variant="inline-block" />
          </section>
        </div>

        <nav
          aria-label="Project links"
          className="mt-20 flex flex-wrap items-center justify-center gap-x-12 gap-y-6 text-slate-400"
        >
          <a
            href="https://github.com/modoterra/echo"
            target="_blank"
            rel="noreferrer"
            aria-label="Echo on GitHub"
            className="inline-flex size-12 items-center justify-center transition hover:text-slate-950"
          >
            <RiGithubFill aria-hidden="true" className="size-9" />
          </a>
          <a
            href="https://www.rust-lang.org/"
            target="_blank"
            rel="noreferrer"
            aria-label="Rust"
            className="inline-flex size-12 items-center justify-center transition hover:text-slate-950"
          >
            <span aria-hidden="true" className="font-mono text-xl font-semibold tracking-normal">
              Rust
            </span>
          </a>
          <a
            href="https://llvm.org/"
            target="_blank"
            rel="noreferrer"
            aria-label="LLVM"
            className="inline-flex size-12 items-center justify-center transition hover:text-slate-950"
          >
            <span aria-hidden="true" className="font-mono text-xl font-semibold tracking-normal">
              LLVM
            </span>
          </a>
          <span
            aria-label="Discord coming later"
            className="inline-flex size-12 cursor-not-allowed items-center justify-center text-slate-300"
            role="img"
          >
            <RiDiscordFill aria-hidden="true" className="size-9" />
          </span>
          <a
            href="https://x.com/hicsfh"
            target="_blank"
            rel="noreferrer"
            aria-label="Hicsfh on X"
            className="inline-flex size-12 items-center justify-center transition hover:text-slate-950"
          >
            <RiTwitterXFill aria-hidden="true" className="size-8" />
          </a>
        </nav>
      </section>
    </main>
  );
}

export default HomePage;
