import { Link } from "@tanstack/react-router";
import { CtaLink } from "./components/cta-link";
import { InstallSnippet } from "./components/install-snippet";

const CLONE_BUILD = `git clone https://github.com/modoterra/echo.git
cd echo
cargo build -p xo
./target/debug/xo --help`;

const FIRST_RUN = `./target/debug/xo run examples/misc/hello.echo
./target/debug/xo run examples/misc/sum_list.echo`;

/**
 * Product install page: honest early-stage source build path.
 * Prebuilt downloads can land here when release artifacts are productized.
 */
export function InstallPage() {
  return (
    <main className="min-h-screen bg-white px-6 pb-24 pt-32 text-slate-950 sm:pt-36">
      <div className="mx-auto w-full max-w-3xl">
        <h1 className="text-3xl font-semibold tracking-normal text-slate-950 sm:text-4xl">
          Install Echo
        </h1>
        <p className="mt-4 text-pretty text-lg leading-8 text-slate-600">
          The public CLI is <span className="font-mono font-semibold text-slate-800">xo</span>.
          Today you build it from source. Prebuilt release binaries will appear here when they are a
          supported product path.
        </p>

        <section className="mt-14">
          <h2 className="text-xl font-semibold tracking-normal text-slate-950 sm:text-2xl">
            Requirements
          </h2>
          <p className="mt-4 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            The supported development baseline is{" "}
            <span className="font-semibold text-slate-800">Linux</span>. Building this repository
            expects Rust (edition 2024),{" "}
            <span className="font-mono font-semibold text-slate-800">clang</span>,{" "}
            <span className="font-mono font-semibold text-slate-800">mold</span>, and{" "}
            <span className="font-mono font-semibold text-slate-800">sccache</span> — the workspace
            Cargo config selects them directly. LLVM is required when codegen is active.
          </p>
        </section>

        <section className="mt-14">
          <h2 className="text-xl font-semibold tracking-normal text-slate-950 sm:text-2xl">
            Build xo
          </h2>
          <p className="mt-4 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            Clone the repository and build the CLI package.
          </p>
          <div className="mt-6">
            <InstallSnippet code={CLONE_BUILD} />
          </div>
        </section>

        <section className="mt-14">
          <h2 className="text-xl font-semibold tracking-normal text-slate-950 sm:text-2xl">
            Run an example
          </h2>
          <p className="mt-4 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            From the checkout root, point{" "}
            <span className="font-mono font-semibold text-slate-800">xo</span> at a sample program.
          </p>
          <div className="mt-6">
            <InstallSnippet code={FIRST_RUN} />
          </div>
        </section>

        <section className="mt-14">
          <h2 className="text-xl font-semibold tracking-normal text-slate-950 sm:text-2xl">
            From here
          </h2>
          <ol className="mt-6 space-y-4 text-base leading-7 text-slate-600">
            <li>
              <span className="font-semibold text-slate-950">1. First program</span> — read the
              minimal shape and the same commands you just ran.
            </li>
            <li>
              <span className="font-semibold text-slate-950">2. Reference</span> — form sheets for
              leaders, Result, structs, and the rest of Echo 2026.
            </li>
            <li>
              <span className="font-semibold text-slate-950">3. Language Spec</span> — edition TOC
              mapping Reference pages to the suite.
            </li>
          </ol>
          <div className="mt-8 flex flex-wrap gap-3">
            <CtaLink to={"/docs/first-program"}>First program</CtaLink>
            <CtaLink to={"/docs"} variant="secondary">
              Reference
            </CtaLink>
            <CtaLink to={"/e26/spec"} variant="secondary">
              Language Spec
            </CtaLink>
          </div>
          <p className="mt-8 text-sm leading-6 text-slate-500">
            Longer project shape and workflow notes live under{" "}
            <Link
              className="font-semibold text-slate-800 underline-offset-4 hover:underline"
              to={"/docs/project" as "/"}
            >
              Project setup
            </Link>
            .
          </p>
        </section>
      </div>
    </main>
  );
}

export default InstallPage;
