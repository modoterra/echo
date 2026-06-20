import {
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Outlet,
  useLocation,
} from "@tanstack/react-router";
import { HomePage } from "./app";

type DocsNavGroup = {
  title: string;
  links: Array<{
    label: string;
    to: string;
    active?: boolean;
    disabled?: boolean;
  }>;
};

type BuiltinDoc = {
  name: string;
  signature: string;
  status: string;
  summary: string;
  gotcha: string;
  echo: string;
  example: string;
  source: string;
};

const builtinDocs: BuiltinDoc[] = [
  {
    name: "strlen",
    signature: "strlen(string $string): int",
    status: "Implemented",
    summary: "Returns the number of bytes in a string.",
    gotcha:
      "Byte length is not character length. A string containing multibyte text can have more bytes than visible characters.",
    echo: "Echo lowers strlen through the PHP builtin ABI and reflects it as string $string returning int.",
    example: `echo strlen("hello") . "\\n";
echo strlen("é") . "\\n";`,
    source: "https://www.php.net/manual/en/function.strlen.php",
  },
  {
    name: "array_is_list",
    signature: "array_is_list(array $array): bool",
    status: "Implemented",
    summary: "Returns true when an array's keys are consecutive integers from 0.",
    gotcha:
      "Associative keys, missing numeric keys, or reordered keys make the array stop being a list.",
    echo: "Echo's current PHP arrays are contiguous vectors, so key-gap and associative-key edge cases are still deferred.",
    example: `echo array_is_list([]) . "\\n";
echo array_is_list([1, 2, 3]) . "\\n";`,
    source: "https://www.php.net/manual/en/function.array-is-list.php",
  },
  {
    name: "function_exists",
    signature: "function_exists(string $function): bool",
    status: "Implemented",
    summary: "Checks whether a named function is available as a function.",
    gotcha:
      "Language constructs are not functions. PHP returns false for names such as echo and include_once.",
    echo: "Echo recognizes supported internal PHP builtin names case-insensitively; user-defined function registry support is deferred.",
    example: `echo function_exists("strlen") . "\\n";
echo function_exists("echo") . "\\n";`,
    source: "https://www.php.net/manual/en/function.function-exists.php",
  },
];

function RootLayout() {
  const location = useLocation();
  const isDocs = location.pathname.startsWith("/docs");

  return (
    <>
      <header
        className={
          isDocs
            ? "fixed inset-x-0 top-0 z-30 border-b border-slate-200/70 bg-white/85 px-6 backdrop-blur"
            : "absolute inset-x-0 top-8 z-20 px-6 sm:top-12"
        }
      >
        <div
          className={
            isDocs
              ? "mx-auto flex h-20 w-full max-w-7xl items-center"
              : "mx-auto flex w-full max-w-[624px] items-center"
          }
        >
          {isDocs ? (
            <Link
              aria-label="Echo home"
              className="mr-10 block w-16 transition opacity-90 hover:opacity-100 lg:mr-[214px] lg:w-20"
              to="/"
            >
              <img alt="Echo" className="h-auto w-full" src="/logo.svg" />
            </Link>
          ) : null}
          <nav
            aria-label="Primary navigation"
            className="flex items-center justify-start gap-8 text-sm font-semibold text-slate-500"
          >
            <Link className="transition hover:text-slate-950" to="/">
              Home
            </Link>
            <Link className="transition hover:text-slate-950" to="/docs">
              Docs
            </Link>
          </nav>
        </div>
      </header>
      <Outlet />
    </>
  );
}

function DocsPage() {
  const navigation: DocsNavGroup[] = [
    {
      title: "Getting Started",
      links: [
        { label: "Installation", to: "/docs", active: true },
        { label: "Quickstart", to: "/docs/quickstart", disabled: true },
        { label: "Source Modes", to: "/docs/source-modes", disabled: true },
      ],
    },
    {
      title: "Language",
      links: [
        { label: "PHP Built-ins", to: "/docs/php-built-ins", disabled: true },
        { label: "PHP Compatibility", to: "/docs/php-compatibility", disabled: true },
        { label: "Strict Mode", to: "/docs/strict-mode", disabled: true },
        { label: "Imports", to: "/docs/imports", disabled: true },
      ],
    },
    {
      title: "Tooling",
      links: [
        { label: "Command Line", to: "/docs/command-line", disabled: true },
        { label: "Language Server", to: "/docs/language-server", disabled: true },
        { label: "Testing", to: "/docs/testing", disabled: true },
        { label: "Source Builds", to: "/docs/source-builds", disabled: true },
      ],
    },
  ];

  const headings = [
    "Meet Echo",
    "Installation",
    "Run a Program",
    "Compile a Program",
    "Project Status",
    "Source Modes",
    "PHP Built-ins",
    "Source Builds",
  ];

  return (
    <main className="min-h-screen bg-white px-6 pb-24 pt-32 text-slate-950">
      <div className="mx-auto grid w-full max-w-7xl grid-cols-1 gap-12 lg:grid-cols-[220px_minmax(0,720px)] xl:grid-cols-[220px_minmax(0,720px)_220px]">
        <aside className="hidden lg:block">
          <nav aria-label="Documentation sections" className="sticky top-32 space-y-10">
            {navigation.map((group) => (
              <section key={group.title}>
                <h2 className="text-sm font-semibold text-slate-950">{group.title}</h2>
                <ul className="mt-5 space-y-3">
                  {group.links.map((link) => (
                    <li key={link.label}>
                      {link.disabled ? (
                        <span className="text-sm leading-6 text-slate-300">{link.label}</span>
                      ) : (
                        <Link
                          className={
                            link.active
                              ? "text-sm font-semibold leading-6 text-slate-950"
                              : "text-sm leading-6 text-slate-500 transition hover:text-slate-950"
                          }
                          to={link.to}
                        >
                          {link.label}
                        </Link>
                      )}
                    </li>
                  ))}
                </ul>
              </section>
            ))}
          </nav>
        </aside>

        <article className="max-w-none">
          <p className="text-sm font-semibold text-slate-500">Getting Started</p>
          <h1 className="mt-6 text-5xl font-semibold tracking-normal text-slate-950">
            Installation
          </h1>

          <section id="meet-echo" className="mt-16 scroll-mt-28">
            <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Meet Echo</h2>
            <p className="mt-6 text-lg leading-8 text-slate-600">
              Echo is a Rust implementation of a PHP superset. Existing PHP should stay familiar,
              while Echo adds compiler tooling, native concurrency, parallel execution, and a path
              toward compiled binaries with predictable performance gains.
            </p>
            <p className="mt-6 text-lg leading-8 text-slate-600">
              The command line entrypoint is <code className="font-mono text-slate-950">xo</code>.
              Echo is early-stage software, so unsupported PHP behavior should fail explicitly
              rather than silently approximate semantics.
            </p>
          </section>

          <section id="installation" className="mt-16 scroll-mt-28">
            <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Installation</h2>
            <p className="mt-6 text-lg leading-8 text-slate-600">
              Install the <code className="font-mono text-slate-950">xo</code> command and keep it
              on your path. The public installer flow is still being designed, so current releases
              are source-built by contributors.
            </p>
            <pre className="mt-8 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
              <code>{`xo --help
xo run app.php
xo build app.php -o app`}</code>
            </pre>
          </section>

          <section id="run-a-program" className="mt-16 scroll-mt-28">
            <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Run a Program</h2>
            <p className="mt-6 text-lg leading-8 text-slate-600">
              Use <code className="font-mono text-slate-950">xo run</code> to execute an
              Echo-compatible PHP file directly from the command line.
            </p>
            <pre className="mt-8 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
              <code>{`xo run examples/hello.php`}</code>
            </pre>
          </section>

          <section id="compile-a-program" className="mt-16 scroll-mt-28">
            <h2 className="text-3xl font-semibold tracking-normal text-slate-950">
              Compile a Program
            </h2>
            <p className="mt-6 text-lg leading-8 text-slate-600">
              Use <code className="font-mono text-slate-950">xo build</code> to compile a supported
              program into a native binary. The current backend lowers through LLVM IR and links
              through the project build path while Echo's native toolchain matures.
            </p>
            <pre className="mt-8 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
              <code>{`xo build examples/hello.php -o /tmp/hello
/tmp/hello`}</code>
            </pre>
          </section>

          <section id="project-status" className="mt-16 scroll-mt-28">
            <h2 className="text-3xl font-semibold tracking-normal text-slate-950">
              Project Status
            </h2>
            <p className="mt-6 text-lg leading-8 text-slate-600">
              Echo currently supports a small but growing PHP-compatible slice across parsing, AST
              generation, LLVM IR codegen, runtime behavior, and CLI execution. The docs should make
              that boundary visible as the language grows.
            </p>
          </section>

          <section id="source-modes" className="mt-16 scroll-mt-28">
            <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Source Modes</h2>
            <p className="mt-6 text-lg leading-8 text-slate-600">
              Files ending in <code className="font-mono text-slate-950">.php</code> use Echo
              superset mode by default. Files ending in{" "}
              <code className="font-mono text-slate-950">.echo</code> or{" "}
              <code className="font-mono text-slate-950">.xo</code> use strict mode by default.
            </p>
          </section>

          <section id="php-built-ins" className="mt-16 scroll-mt-28">
            <p className="text-sm font-semibold text-slate-500">Language</p>
            <h2 className="mt-4 text-3xl font-semibold tracking-normal text-slate-950">
              PHP Built-ins
            </h2>
            <p className="mt-6 text-lg leading-8 text-slate-600">
              Echo documents PHP built-ins as executable compatibility contracts: signature,
              behavior, common trap, Echo status, and the fixture-backed edge that matters. The
              first pass starts with the built-ins already reflected and lowered by Echo.
            </p>

            <div className="mt-10 divide-y divide-slate-200 border-y border-slate-200">
              {builtinDocs.map((builtin) => (
                <section key={builtin.name} className="py-8">
                  <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                    <div>
                      <h3 className="font-mono text-2xl font-semibold text-slate-950">
                        {builtin.name}
                      </h3>
                      <p className="mt-3 font-mono text-sm text-slate-500">{builtin.signature}</p>
                    </div>
                    <span className="w-fit rounded-full bg-cyan-50 px-3 py-1 text-xs font-semibold text-cyan-700">
                      {builtin.status}
                    </span>
                  </div>

                  <dl className="mt-7 grid gap-6 text-base leading-7 sm:grid-cols-2">
                    <div>
                      <dt className="text-sm font-semibold text-slate-950">Behavior</dt>
                      <dd className="mt-2 text-slate-600">{builtin.summary}</dd>
                    </div>
                    <div>
                      <dt className="text-sm font-semibold text-slate-950">Watch For</dt>
                      <dd className="mt-2 text-slate-600">{builtin.gotcha}</dd>
                    </div>
                    <div>
                      <dt className="text-sm font-semibold text-slate-950">Echo Status</dt>
                      <dd className="mt-2 text-slate-600">{builtin.echo}</dd>
                    </div>
                    <div>
                      <dt className="text-sm font-semibold text-slate-950">Manual Source</dt>
                      <dd className="mt-2">
                        <a
                          className="text-slate-500 underline decoration-slate-300 underline-offset-4 transition hover:text-slate-950"
                          href={builtin.source}
                          rel="noreferrer"
                          target="_blank"
                        >
                          php.net
                        </a>
                      </dd>
                    </div>
                  </dl>

                  <pre className="mt-7 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
                    <code>{builtin.example}</code>
                  </pre>
                </section>
              ))}
            </div>
          </section>

          <section id="source-builds" className="mt-16 scroll-mt-28">
            <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Source Builds</h2>
            <p className="mt-6 text-lg leading-8 text-slate-600">
              Contributors can build the current command line from source. Full workspace builds
              require Rust, LLVM 22, clang, and PHP for compatibility fixture generation.
            </p>
            <pre className="mt-8 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
              <code>{`git clone https://github.com/modoterra/echo.git
cd echo
cargo build -p xo
cargo test --workspace
cargo run -p xo -- run examples/hello.php`}</code>
            </pre>
          </section>
        </article>

        <aside className="hidden xl:block">
          <nav aria-label="On this page" className="sticky top-32 border-l border-slate-200 pl-6">
            <h2 className="text-sm font-semibold text-slate-950">On this page</h2>
            <ul className="mt-5 space-y-3">
              {headings.map((heading) => (
                <li key={heading}>
                  <a
                    className="text-sm leading-6 text-slate-500 transition hover:text-slate-950"
                    href={`#${heading.toLowerCase().replaceAll(" ", "-")}`}
                  >
                    {heading}
                  </a>
                </li>
              ))}
            </ul>
          </nav>
        </aside>
      </div>
    </main>
  );
}

const rootRoute = createRootRoute({
  component: RootLayout,
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: HomePage,
});

const docsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs",
  component: DocsPage,
});

const routeTree = rootRoute.addChildren([indexRoute, docsRoute]);

export const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
