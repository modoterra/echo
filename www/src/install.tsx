import { Link } from "@tanstack/react-router";
import { CtaLink } from "./components/cta-link";
import { InstallSnippet } from "./components/install-snippet";
import {
  currentPrereleaseTag,
  currentPrereleaseAssets,
  currentPrereleaseUrl,
  releasesIndexUrl,
} from "./lib/current-release";

const PREBUILT_INSTALL = `# Newest published prerelease for this machine (linux-x86_64 / macos-arm64)
curl -fsSL https://raw.githubusercontent.com/modoterra/echo/main/scripts/install.sh \\
  | bash -s -- from-release

# Pin this tag
# … | bash -s -- from-release ${currentPrereleaseTag}

xo --help
xo doctor 2>/dev/null || true`;

const CLONE_BUILD = `git clone https://github.com/modoterra/echo.git
cd echo
cargo build -p xo
./target/debug/xo --help`;

const USER_INSTALL = `# From the checkout: release build + XDG install + ~/.local/bin/xo
./scripts/install.sh
./scripts/install.sh doctor

# Newest published prerelease, or pin a tag
./scripts/install.sh from-release
./scripts/install.sh from-release ${currentPrereleaseTag}

# Upgrade (keeps prior toolchain dirs)
./scripts/install.sh upgrade

# Uninstall toolchain (add --purge to also clear $XO_HOME packages)
./scripts/uninstall.sh`;

const FIRST_RUN = `# After from-release / install.sh, xo is on PATH
xo run examples/misc/hello.echo

# Or from a debug build in a checkout
./target/debug/xo run examples/misc/hello.echo
./target/debug/xo run examples/misc/sum_list.echo`;

/**
 * Product install page: current prerelease assets + source build fallback.
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
          Published builds are prereleases. The current tag is{" "}
          <a
            className="font-mono font-semibold text-slate-800 underline-offset-4 hover:underline"
            href={currentPrereleaseUrl}
          >
            {currentPrereleaseTag}
          </a>
          . Take a prebuilt when you only need to run programs. Build from source when you edit the
          compiler.
        </p>

        <section className="mt-14">
          <h2 className="text-xl font-semibold tracking-normal text-slate-950 sm:text-2xl">
            Prebuilt (recommended)
          </h2>
          <p className="mt-4 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            <span className="font-mono text-slate-800">{currentPrereleaseTag}</span> ships{" "}
            {currentPrereleaseAssets.map((asset, index) => (
              <span key={asset.archive}>
                {index > 0 ? " and " : ""}
                <span className="font-mono text-slate-800">{asset.archive}</span>
              </span>
            ))}
            . The script downloads <span className="font-mono text-slate-800">xo</span>,{" "}
            <span className="font-mono text-slate-800">libecho_runtime.a</span>, and{" "}
            <span className="font-mono text-slate-800">std/</span> for that host, then links{" "}
            <span className="font-mono font-semibold text-slate-800">~/.local/bin/xo</span>.
          </p>
          <p className="mt-4 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            <span className="font-mono text-slate-800">from-release</span> with no tag installs the
            newest published prerelease. Pass a tag to pin. GitHub{" "}
            <span className="font-mono text-slate-800">/releases/latest</span> only resolves a
            non-prerelease and 404s today. See the{" "}
            <a
              className="font-semibold text-slate-800 underline-offset-4 hover:underline"
              href={releasesIndexUrl}
            >
              releases list
            </a>
            .
          </p>
          <ul className="mt-6 space-y-2 text-base leading-7 text-slate-600">
            {currentPrereleaseAssets.map((asset) => (
              <li key={asset.artifact}>
                <span className="font-mono font-semibold text-slate-800">{asset.archive}</span>
                {" · "}
                {asset.host}
              </li>
            ))}
          </ul>
          <div className="mt-6">
            <InstallSnippet code={PREBUILT_INSTALL} />
          </div>
        </section>

        <section className="mt-14">
          <h2 className="text-xl font-semibold tracking-normal text-slate-950 sm:text-2xl">
            Requirements
          </h2>
          <p className="mt-4 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            To <span className="font-semibold text-slate-800">run</span> programs you need{" "}
            <span className="font-mono font-semibold text-slate-800">clang</span> on{" "}
            <span className="font-semibold text-slate-800">PATH</span> for AOT link. Building this
            repository also expects Rust (edition 2024),{" "}
            <span className="font-mono font-semibold text-slate-800">mold</span>, and{" "}
            <span className="font-mono font-semibold text-slate-800">sccache</span>. The workspace
            Cargo config selects those tools directly. Compiling the toolchain needs LLVM as well.
          </p>
        </section>

        <section className="mt-14">
          <h2 className="text-xl font-semibold tracking-normal text-slate-950 sm:text-2xl">
            Build from source
          </h2>
          <p className="mt-4 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            Clone the repository and build the CLI package when you develop Echo itself.
          </p>
          <div className="mt-6">
            <InstallSnippet code={CLONE_BUILD} />
          </div>
        </section>

        <section className="mt-14">
          <h2 className="text-xl font-semibold tracking-normal text-slate-950 sm:text-2xl">
            Install to PATH (XDG)
          </h2>
          <p className="mt-4 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            From a checkout, install co-located{" "}
            <span className="font-mono font-semibold text-slate-800">xo</span> and{" "}
            <span className="font-mono font-semibold text-slate-800">std</span> under XDG data, link{" "}
            <span className="font-mono font-semibold text-slate-800">~/.local/bin/xo</span>, and
            create package cache and state dirs. Upgrade flips{" "}
            <span className="font-mono text-slate-800">current</span> without wiping packages. The
            checkout file <span className="font-mono text-slate-800">docs/install.md</span> records
            install layout details.
          </p>
          <div className="mt-6">
            <InstallSnippet code={USER_INSTALL} />
          </div>
        </section>

        <section className="mt-14">
          <h2 className="text-xl font-semibold tracking-normal text-slate-950 sm:text-2xl">
            Run an example
          </h2>
          <p className="mt-4 text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            Point <span className="font-mono font-semibold text-slate-800">xo</span> at a sample
            program from a checkout, or at any path after install.
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
              <span className="font-semibold text-slate-950">1. First program</span>: the minimal
              shape and the same commands you just ran.
            </li>
            <li>
              <span className="font-semibold text-slate-950">2. Reference</span>: form sheets for
              leaders, Result, structs, and the rest of Echo 2026.
            </li>
            <li>
              <span className="font-semibold text-slate-950">3. Language Spec</span>: edition TOC
              that maps Reference pages to the suite.
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
