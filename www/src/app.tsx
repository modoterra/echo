import { useEffect, type ReactNode } from "react";
import { RiDiscordFill, RiGithubFill, RiTwitterXFill } from "@remixicon/react";
import { Link } from "@tanstack/react-router";
import { CtaLink } from "./components/cta-link";
import { EchoCode } from "./components/echo-code";
import { GradientBackground } from "./components/gradient-background";
import { InstallSnippet } from "./components/install-snippet";
import { applyRandomization } from "./lib/randomize-bg";

const HERO_ECHO = `/ std/io

$ xs = [1, 2, 3]
~ sum = 0
* x : xs {
    ~ sum = sum + x
}
io.print("sum={sum}")
`;

const LEADERS_SNIP = `$ name = expr
~ total = total + x
? ready {
    ^ "ok"
}
* item : items {
    io.print(item)
}`;

const ERRORS_SNIP = `! "not found"

| result {
    $ value {
        ^ value
    }
    ! err {
        ^ fallback
    }
}`;

const TOOLING_SNIP = `xo check app.echo
xo run app.echo
xo build app.echo -o app`;

const QUICK_INSTALL = `git clone https://github.com/modoterra/echo.git
cd echo
cargo build -p xo`;

export function HomePage() {
  useEffect(() => {
    applyRandomization();
  }, []);

  return (
    <main className="hero flex min-h-screen bg-white px-6 pb-24 pt-28 text-slate-950 sm:pt-32">
      <GradientBackground />
      <div className="relative mx-auto flex w-full max-w-5xl flex-col">
        {/* ── Hero ─────────────────────────────────────────────── */}
        <section className="flex flex-col items-center pt-6 text-center sm:pt-10">
          <p className="font-mono text-xs font-semibold tracking-wide text-slate-500">
            Echo 2026 · early · open source
          </p>
          <h1 className="mt-5 max-w-3xl text-balance text-[clamp(1.5rem,4.2vw,2.5rem)] font-semibold leading-tight tracking-normal text-slate-950">
            Echo is a compiled language with leaders instead of keywords.
          </h1>
          <p className="mx-auto mt-4 max-w-2xl text-pretty text-lg leading-8 text-slate-600 sm:text-xl">
            Write clear programs. Check them. Ship native binaries with{" "}
            <span className="font-mono font-semibold text-slate-800">xo</span>.
          </p>
          <div className="mt-8 flex flex-wrap items-center justify-center gap-3">
            <CtaLink to="/install">Install Echo</CtaLink>
            <CtaLink to={"/docs/first-program"} variant="secondary">
              First program
            </CtaLink>
          </div>

          <div className="mt-12 w-full max-w-3xl text-left">
            <EchoCode
              aria-label="Echo language example"
              code={HERO_ECHO}
              language="echo"
              variant="hero"
            />
            <div className="mt-3 overflow-hidden rounded-lg border border-slate-200/80 bg-slate-950 px-5 py-4 font-mono text-sm leading-7 text-slate-100 sm:px-7">
              <p className="text-xs font-semibold tracking-wide text-slate-400">Output</p>
              <p className="mt-2 text-slate-100">
                <span className="text-slate-500">$ </span>
                xo run sum.echo
              </p>
              <p className="text-emerald-300">sum=6</p>
            </div>
          </div>
        </section>

        {/* ── Pillars ──────────────────────────────────────────── */}
        <section className="mt-24 sm:mt-28" aria-labelledby="pillars-heading">
          <h2 id="pillars-heading" className="sr-only">
            Why Echo
          </h2>
          <div className="grid gap-6 md:grid-cols-3">
            <PillarCard
              title="Leaders, not keywords"
              body={
                <>
                  A statement starts with a glyph: <Mono>$</Mono> binds, <Mono>~</Mono> mutates,{" "}
                  <Mono>?</Mono> branches, <Mono>*</Mono> loops, <Mono>^</Mono> returns. The rest is
                  ordinary expressions.
                </>
              }
              code={LEADERS_SNIP}
              href={"/docs/leaders"}
              linkLabel="Leaders reference"
            />
            <PillarCard
              title="Errors are values"
              body={
                <>
                  <Mono>!</Mono> returns an error from a function; it does not abort the process.
                  You match the result, or the program does not compile. Same idea for optionals.
                </>
              }
              code={ERRORS_SNIP}
              href={"/docs/result-option"}
              linkLabel="Result and option"
            />
            <PillarCard
              title="Small loop, native output"
              body={
                <>
                  Imports stay module-scoped. I/O comes from <Mono>std</Mono>, not free globals.{" "}
                  <Mono>xo run</Mono> and <Mono>xo build</Mono> are the everyday loop.
                </>
              }
              code={TOOLING_SNIP}
              language="shellscript"
              href={"/docs/toolchain/commands"}
              linkLabel="Toolchain commands"
            />
          </div>
        </section>

        {/* ── Tooling loop ─────────────────────────────────────── */}
        <section className="mt-24 sm:mt-28" aria-labelledby="tooling-heading">
          <h2
            id="tooling-heading"
            className="text-2xl font-semibold tracking-normal text-slate-950 sm:text-3xl"
          >
            One CLI for the loop
          </h2>
          <p className="mt-4 max-w-2xl text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            Check for diagnostics, run for the fast cycle, build when you want a native binary.
          </p>
          <ol className="mt-8 grid gap-4 sm:grid-cols-3">
            <ToolStep step="1" title="check" detail="Resolve and type-check the graph." />
            <ToolStep step="2" title="run" detail="Compile and execute via LLVM." />
            <ToolStep step="3" title="build" detail="Emit a native binary you can ship." />
          </ol>
          <div className="mt-8 max-w-2xl">
            <InstallSnippet code={QUICK_INSTALL} label="Get started from source" />
            <p className="mt-4 text-sm leading-6 text-slate-500">
              Full requirements and next steps on{" "}
              <Link
                className="font-semibold text-slate-800 underline-offset-4 hover:underline"
                to={"/install" as "/"}
              >
                Install
              </Link>
              .
            </p>
          </div>
        </section>

        {/* ── Learn path ───────────────────────────────────────── */}
        <section className="mt-24 sm:mt-28" aria-labelledby="learn-heading">
          <h2
            id="learn-heading"
            className="text-2xl font-semibold tracking-normal text-slate-950 sm:text-3xl"
          >
            Where to go next
          </h2>
          <div className="mt-8 grid gap-4 sm:grid-cols-3">
            <LearnCard
              title="First program"
              body="Minimal runnable shape and the xo commands that drive it."
              to={"/docs/first-program"}
            />
            <LearnCard
              title="Reference"
              body="Form-by-form rules for the Echo 2026 language surface."
              to={"/docs"}
            />
            <LearnCard
              title="Echo 2026"
              body="Edition home: public Spec framing and the executable suite contract."
              to={"/e26"}
            />
          </div>
        </section>

        {/* ── Edition / trust ──────────────────────────────────── */}
        <section
          className="mt-24 rounded-2xl border border-slate-200 bg-white/70 px-6 py-10 backdrop-blur-sm sm:mt-28 sm:px-10"
          aria-labelledby="edition-heading"
        >
          <h2
            id="edition-heading"
            className="text-2xl font-semibold tracking-normal text-slate-950 sm:text-3xl"
          >
            Echo 2026
          </h2>
          <p className="mt-4 max-w-2xl text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            Echo 2026 is the current language edition and the canonical public Language Spec on this
            site. The form-by-form rules live in the Reference; the machine-checked contract is the{" "}
            <span className="font-mono font-semibold text-slate-800">echo26/</span> suite, driven by{" "}
            <span className="font-mono font-semibold text-slate-800">e26</span>.
          </p>
          <p className="mt-4 max-w-2xl text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            The project is early and actively implemented. Built with Rust and LLVM. Open source.
          </p>
          <div className="mt-8 flex flex-wrap gap-3">
            <CtaLink to={"/e26"} variant="secondary">
              Open Echo 2026
            </CtaLink>
            <CtaLink to={"/book"} variant="ghost">
              Read the Book
            </CtaLink>
          </div>
        </section>

        {/* ── Social / project links ───────────────────────────── */}
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
      </div>
    </main>
  );
}

function Mono({ children }: { children: string }) {
  return <span className="font-mono font-semibold text-slate-800">{children}</span>;
}

function PillarCard({
  title,
  body,
  code,
  language = "echo",
  href,
  linkLabel,
}: {
  title: string;
  body: ReactNode;
  code: string;
  language?: "echo" | "shellscript";
  href: string;
  linkLabel: string;
}) {
  return (
    <article className="flex flex-col rounded-xl border border-slate-200/90 bg-white/80 p-5 shadow-sm backdrop-blur-sm sm:p-6">
      <h3 className="text-lg font-semibold tracking-normal text-slate-950 sm:text-xl">{title}</h3>
      <p className="mt-3 flex-1 text-pretty text-sm leading-6 text-slate-600 sm:text-base sm:leading-7">
        {body}
      </p>
      <div className="mt-5">
        <EchoCode
          code={code}
          language={language}
          variant="inline-block"
          className="overflow-x-auto rounded-lg border border-slate-200 bg-slate-50"
        />
      </div>
      <Link
        className="mt-5 text-sm font-semibold text-slate-800 underline-offset-4 hover:underline"
        to={href as "/"}
      >
        {linkLabel} →
      </Link>
    </article>
  );
}

function ToolStep({ step, title, detail }: { step: string; title: string; detail: string }) {
  return (
    <li className="rounded-xl border border-slate-200 bg-white/80 px-5 py-5">
      <p className="font-mono text-xs font-semibold tracking-wide text-slate-400">{step}</p>
      <p className="mt-2 font-mono text-lg font-semibold text-slate-950">xo {title}</p>
      <p className="mt-2 text-sm leading-6 text-slate-600">{detail}</p>
    </li>
  );
}

function LearnCard({ title, body, to }: { title: string; body: string; to: string }) {
  return (
    <Link
      to={to as "/"}
      className="group flex flex-col rounded-xl border border-slate-200 bg-white/80 p-6 transition hover:border-slate-300 hover:shadow-sm"
    >
      <h3 className="text-lg font-semibold text-slate-950 group-hover:text-slate-800">{title}</h3>
      <p className="mt-3 text-sm leading-6 text-slate-600">{body}</p>
      <span className="mt-5 text-sm font-semibold text-slate-800">Open →</span>
    </Link>
  );
}

export default HomePage;
