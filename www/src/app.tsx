import { RiGithubFill } from "@remixicon/react";
import { Link } from "@tanstack/react-router";
import { CodeStage } from "./components/code-stage";
import { CtaLink } from "./components/cta-link";
import { EchoCode } from "./components/echo-code";
import { GradientBackground } from "./components/gradient-background";
import { InstallSnippet } from "./components/install-snippet";
import { HOME_DEMOS } from "./lib/home-demos";

const HERO_ECHO = `/ std/io

$ xs = [1, 2, 3]
~ sum = 0
* x : xs {
    ~ sum = sum + x
}
io.print("sum={sum}")
`;

const QUICK_INSTALL = `git clone https://github.com/modoterra/echo.git
cd echo
./scripts/install.sh
./scripts/install.sh doctor`;

const PROOF_POINTS = [
  { value: "Echo 2026", label: "public language edition" },
  { value: "AOT + JIT", label: "one LLVM backend" },
  { value: "Rust", label: "compiler and runtime" },
  { value: "Open source", label: "implementation and spec" },
];

const TOOLCHAIN_STEPS = [
  {
    command: "xo check",
    detail: "Resolves and type-checks the full module graph before anything runs.",
  },
  {
    command: "xo run",
    detail: "Compiles and executes through the shared LLVM pipeline.",
  },
  {
    command: "xo build",
    detail: "Emits a native executable from that same compiler and runtime.",
  },
];

const EDITION_LINKS = [
  {
    eyebrow: "Learn",
    title: "First program",
    detail: "Write the smallest complete Echo program, then check and run it.",
    to: "/docs/first-program",
  },
  {
    eyebrow: "Reference",
    title: "Language forms",
    detail: "Form-by-form rules for leaders, values, structs, and tasks.",
    to: "/docs",
  },
  {
    eyebrow: "Specification",
    title: "Echo 2026",
    detail: "Public language law mapped to the machine-checked conformance suite.",
    to: "/e26/spec",
  },
];

export function HomePage() {
  return (
    <main className="overflow-hidden bg-white text-slate-950">
      <section className="home-hero px-6 pb-16 pt-28 sm:pb-20 sm:pt-32 lg:pb-24 lg:pt-28">
        <GradientBackground />
        <div className="relative mx-auto grid w-full max-w-7xl items-center gap-14 lg:grid-cols-[minmax(0,0.9fr)_minmax(560px,1.1fr)] lg:gap-12">
          <div className="max-w-2xl text-center lg:text-left">
            <h1 className="text-balance font-display text-[clamp(2.25rem,11vw,4.25rem)] font-bold leading-[0.98] tracking-[-0.055em] text-slate-950">
              Echo is a compiled language. Leaders mark each statement.
            </h1>
            <p className="mx-auto mt-7 max-w-xl text-pretty text-lg leading-8 text-slate-600 sm:text-xl lg:mx-0">
              Each statement begins with a leader glyph. The{" "}
              <span className="font-mono font-semibold text-slate-900">xo</span> CLI checks a program
              and emits a native binary from the same pipeline.
            </p>
            <div className="mt-9 flex flex-wrap items-center justify-center gap-3 lg:justify-start">
              <CtaLink className="min-w-32" to="/install">
                Install Echo
              </CtaLink>
              <CtaLink className="min-w-32" to="/docs/first-program" variant="secondary">
                First program
              </CtaLink>
            </div>
            <p className="mt-5 text-sm text-slate-500">
              Install from source for now. Language and toolchain keep moving.
            </p>

            <div className="mx-auto mt-10 max-w-lg overflow-hidden rounded-2xl border border-slate-800 bg-slate-950 text-left font-mono shadow-2xl shadow-slate-950/15 md:hidden">
              <div className="flex items-center justify-between border-b border-slate-800 px-4 py-2.5 text-[0.7rem] font-semibold tracking-wide text-slate-500">
                <span>terminal</span>
                <span className="text-emerald-400">native run</span>
              </div>
              <div className="px-4 py-4 text-sm leading-7 text-slate-100">
                <p>
                  <span className="text-violet-400">$</span> xo run sum.echo
                </p>
                <p className="text-emerald-300">sum=6</p>
              </div>
            </div>
          </div>

          <HeroCompilerStage />
        </div>
      </section>

      <ProofRail />

      <div className="mx-auto w-full max-w-7xl px-6 pb-28 sm:pb-36">
        <CodeStage
          demos={HOME_DEMOS}
          title="The language surface stays in sight."
          subtitle="Each leader shows the role of a statement before you read the rest of the line. Open a sample, run it with xo, and read the output beside the form."
        />

        <ToolchainStory />
        <EditionStory />
        <ClosingCallToAction />
      </div>
    </main>
  );
}

function HeroCompilerStage() {
  return (
    <figure
      className="echo-hero-stage relative hidden min-h-[520px] md:block"
      aria-label="An Echo source file moving through xo into a native executable"
    >
      <div className="echo-stage-glyphs" aria-hidden="true">
        <span>$</span>
        <span>~</span>
        <span>?</span>
        <span>*</span>
        <span>^</span>
      </div>
      <div className="echo-stage-rail" aria-hidden="true" />

      <div className="echo-stage-source overflow-hidden rounded-2xl border border-slate-200/90 bg-white/95 shadow-2xl shadow-indigo-950/15 backdrop-blur">
        <div className="flex items-center justify-between border-b border-slate-200 bg-slate-50/80 px-4 py-3">
          <div className="flex items-center gap-1.5" aria-hidden="true">
            <span className="size-2.5 rounded-full bg-rose-300" />
            <span className="size-2.5 rounded-full bg-amber-300" />
            <span className="size-2.5 rounded-full bg-emerald-300" />
          </div>
          <span className="font-mono text-xs font-semibold text-slate-500">sum.echo</span>
          <span className="rounded-full bg-violet-100 px-2 py-1 font-mono text-[0.65rem] font-semibold text-violet-700">
            source
          </span>
        </div>
        <EchoCode
          aria-label="Echo source example"
          className="overflow-x-auto bg-white/70"
          code={HERO_ECHO}
          language="echo"
          variant="inline-block"
        />
      </div>

      <div className="echo-stage-command overflow-hidden rounded-xl border border-slate-800 bg-slate-950 font-mono shadow-xl shadow-slate-950/20">
        <div className="flex items-center justify-between border-b border-slate-800 px-4 py-2 text-[0.65rem] font-semibold tracking-wide text-slate-500">
          <span>xo</span>
          <span className="text-cyan-300">check → run → build</span>
        </div>
        <div className="px-4 py-3 text-sm leading-6 text-slate-100">
          <p>
            <span className="text-violet-400">$</span> xo build sum.echo -o sum
          </p>
          <p className="text-emerald-300">built ./sum</p>
        </div>
      </div>

      <div className="echo-stage-artifact rounded-2xl border border-violet-200 bg-white p-4 shadow-xl shadow-indigo-950/10">
        <div className="flex items-center gap-3">
          <span className="echo-artifact-mark inline-flex size-12 items-center justify-center rounded-xl font-display text-xl font-bold text-white">
            xo
          </span>
          <span>
            <span className="block font-mono text-sm font-semibold text-slate-950">./sum</span>
            <span className="mt-0.5 block text-xs text-slate-500">native executable</span>
          </span>
        </div>
      </div>

      <figcaption className="sr-only">
        Echo source is checked, run, and built by xo into a native executable.
      </figcaption>
    </figure>
  );
}

function ProofRail() {
  return (
    <section className="relative border-y border-slate-200/80 bg-white/90" aria-label="Echo facts">
      <dl className="mx-auto grid w-full max-w-7xl grid-cols-2 px-6 lg:grid-cols-4">
        {PROOF_POINTS.map((point, index) => (
          <div
            key={point.value}
            className={`py-6 sm:py-7 ${index % 2 === 0 ? "pr-4" : "border-l border-slate-200 pl-4"} ${index > 1 ? "border-t border-slate-200 lg:border-t-0" : ""} lg:border-l lg:px-8 lg:first:border-l-0 lg:first:pl-0 lg:last:pr-0`}
          >
            <dt className="font-display text-lg font-bold tracking-tight text-slate-950 sm:text-xl">
              {point.value}
            </dt>
            <dd className="mt-1 text-sm leading-5 text-slate-500">{point.label}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

function ToolchainStory() {
  return (
    <section className="relative mt-28 overflow-hidden rounded-[2rem] bg-slate-950 px-6 py-10 text-white shadow-2xl shadow-slate-950/15 sm:mt-36 sm:px-10 sm:py-14 lg:px-14 lg:py-16">
      <div className="toolchain-glow" aria-hidden="true" />
      <div className="relative grid gap-12 lg:grid-cols-[minmax(0,0.85fr)_minmax(520px,1.15fr)] lg:gap-16">
        <div className="min-w-0">
          <p className="font-mono text-xs font-semibold tracking-[0.16em] text-cyan-300">
            SOURCE → NATIVE
          </p>
          <h2 className="mt-5 max-w-lg text-balance font-display text-4xl font-bold leading-[1.02] tracking-[-0.045em] sm:text-5xl">
            One tool for the whole loop.
          </h2>
          <p className="mt-6 max-w-lg text-pretty text-base leading-7 text-slate-300 sm:text-lg sm:leading-8">
            The check path is the product path. CLI, language server, AOT compiler, and JIT share
            one pipeline, so every host sees the same language rules.
          </p>
          <div className="mt-8 flex flex-wrap gap-3">
            <CtaLink to="/docs/toolchain/commands">Toolchain commands</CtaLink>
            <CtaLink
              className="border-slate-700 bg-slate-900 text-slate-100 hover:border-slate-500 hover:text-white"
              to="/docs/first-program"
              variant="secondary"
            >
              Run a program
            </CtaLink>
          </div>
        </div>

        <ol className="relative divide-y divide-slate-800 border-y border-slate-800">
          {TOOLCHAIN_STEPS.map((step, index) => (
            <li
              key={step.command}
              className="grid gap-3 py-6 sm:grid-cols-[3rem_10rem_1fr] sm:items-start"
            >
              <span className="font-mono text-xs font-semibold text-slate-600">0{index + 1}</span>
              <span className="font-mono text-base font-semibold text-white">{step.command}</span>
              <span className="text-sm leading-6 text-slate-400">{step.detail}</span>
            </li>
          ))}
        </ol>
      </div>

      <div className="relative mt-12 grid gap-8 border-t border-slate-800 pt-10 lg:grid-cols-[minmax(0,0.7fr)_minmax(520px,1.3fr)] lg:gap-14">
        <div className="min-w-0">
          <p className="font-mono text-xs font-semibold tracking-[0.16em] text-violet-300">
            INSTALL XO
          </p>
          <h3 className="mt-4 max-w-md font-display text-3xl font-bold leading-tight tracking-[-0.035em] text-white">
            Put the toolchain on your PATH.
          </h3>
          <p className="mt-4 max-w-lg text-sm leading-6 text-slate-400 sm:text-base sm:leading-7">
            From a checkout, the installer builds a release and places{" "}
            <span className="font-mono text-slate-200">xo</span> next to{" "}
            <span className="font-mono text-slate-200">std</span>. By default it links{" "}
            <span className="font-mono text-slate-200">xo</span> into{" "}
            <span className="font-mono text-slate-200">~/.local/bin</span>.
          </p>
          <Link
            className="mt-5 inline-flex text-sm font-semibold text-violet-300 transition hover:text-violet-200"
            to="/install"
          >
            Requirements and full instructions →
          </Link>
        </div>

        <div className="min-w-0">
          <InstallSnippet code={QUICK_INSTALL} label="Install xo from a checkout" />
          <div className="mt-3 grid gap-3 sm:grid-cols-2">
            <div className="rounded-xl border border-slate-800 bg-slate-900/70 px-4 py-3">
              <p className="text-xs font-semibold text-slate-500">Upgrade</p>
              <code className="mt-2 block overflow-x-auto font-mono text-xs text-slate-200">
                ./scripts/install.sh upgrade
              </code>
            </div>
            <div className="rounded-xl border border-slate-800 bg-slate-900/70 px-4 py-3">
              <p className="text-xs font-semibold text-slate-500">Uninstall</p>
              <code className="mt-2 block overflow-x-auto font-mono text-xs text-slate-200">
                ./scripts/uninstall.sh
              </code>
            </div>
          </div>
          <p className="mt-4 text-xs leading-5 text-slate-500">
            If needed, add <span className="font-mono text-slate-400">~/.local/bin</span> to your{" "}
            <span className="font-mono text-slate-400">PATH</span>.
          </p>
        </div>
      </div>
    </section>
  );
}

function EditionStory() {
  return (
    <section className="mt-28 grid gap-14 sm:mt-36 lg:grid-cols-[minmax(0,0.9fr)_minmax(520px,1.1fr)] lg:items-start lg:gap-20">
      <div className="lg:sticky lg:top-32">
        <p className="font-mono text-xs font-semibold tracking-[0.16em] text-violet-600">
          LANGUAGE LAW
        </p>
        <h2 className="mt-5 max-w-xl text-balance font-display text-4xl font-bold leading-[1.02] tracking-[-0.045em] sm:text-5xl">
          Echo 2026 is written down and executable.
        </h2>
        <p className="mt-6 max-w-xl text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
          The public Reference states the language rules. The Echo 2026 suite runs those rules as
          fixtures against a candidate binary, so the edition stays machine-checked.
        </p>
        <div className="mt-8 flex flex-wrap gap-3">
          <CtaLink to="/e26/spec">Open the Language Spec</CtaLink>
          <CtaLink to="/e26" variant="secondary">
            Edition overview
          </CtaLink>
        </div>
      </div>

      <div className="border-t border-slate-200">
        {EDITION_LINKS.map((item, index) => (
          <Link
            key={item.title}
            className="group grid gap-4 border-b border-slate-200 py-7 transition sm:grid-cols-[5rem_1fr_auto] sm:items-center sm:py-8"
            to={item.to as "/"}
          >
            <span className="font-mono text-xs font-semibold text-slate-400">0{index + 1}</span>
            <span>
              <span className="block text-xs font-semibold uppercase tracking-[0.14em] text-violet-600">
                {item.eyebrow}
              </span>
              <span className="mt-2 block font-display text-2xl font-bold tracking-tight text-slate-950">
                {item.title}
              </span>
              <span className="mt-2 block max-w-lg text-sm leading-6 text-slate-500">
                {item.detail}
              </span>
            </span>
            <span
              className="hidden size-11 items-center justify-center rounded-full border border-slate-200 text-lg text-slate-500 transition group-hover:border-violet-300 group-hover:bg-violet-50 group-hover:text-violet-700 sm:inline-flex"
              aria-hidden="true"
            >
              →
            </span>
          </Link>
        ))}
      </div>
    </section>
  );
}

function ClosingCallToAction() {
  return (
    <section className="home-closing relative mt-28 overflow-hidden rounded-[2rem] border border-violet-200 px-6 py-14 text-center sm:mt-36 sm:px-12 sm:py-20">
      <div className="home-closing-glyphs" aria-hidden="true">
        $ ~ ? * ^ !
      </div>
      <div className="relative mx-auto max-w-3xl">
        <p className="font-mono text-xs font-semibold tracking-[0.16em] text-violet-700">
          START WITH XO
        </p>
        <h2 className="mt-5 text-balance font-display text-4xl font-bold leading-[1.02] tracking-[-0.045em] sm:text-6xl">
          Install xo and run a program.
        </h2>
        <p className="mx-auto mt-6 max-w-xl text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
          Build the toolchain, open the first program page, and keep the Reference next to the code
          while you edit.
        </p>
        <div className="mt-9 flex flex-wrap justify-center gap-3">
          <CtaLink to="/install">Install Echo</CtaLink>
          <CtaLink to="/docs/first-program" variant="secondary">
            First program
          </CtaLink>
          <a
            className="inline-flex items-center justify-center gap-2 rounded-lg px-4 py-2.5 text-sm font-semibold text-slate-600 transition hover:text-slate-950 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-violet-300"
            href="https://github.com/modoterra/echo"
            rel="noreferrer"
            target="_blank"
          >
            <RiGithubFill className="size-5" aria-hidden="true" />
            GitHub
          </a>
        </div>
      </div>
    </section>
  );
}

export default HomePage;
