import {
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Outlet,
  useLocation,
} from "@tanstack/react-router";
import type { ReactNode } from "react";
import { HomePage } from "./app";

type DocsNavGroup = {
  title: string;
  links: Array<{
    label: string;
    to: string;
    disabled?: boolean;
  }>;
};

type DocsShellProps = {
  category: string;
  title: string;
  headings: string[];
  children: ReactNode;
};

type BuiltinDoc = {
  name: string;
  signature: string;
  description: string;
  example: string;
};

const builtinDocs: BuiltinDoc[] = [
  {
    name: "strlen",
    signature: "strlen(string $string): int",
    description:
      "Returns the number of bytes in a string. This is byte length, not character length, so multibyte text can be longer than the number of visible characters.",
    example: `let $word = "hello"
let $accented = "é"

echo strlen($word) . "\\n"
echo strlen($accented) . "\\n"`,
  },
  {
    name: "array_is_list",
    signature: "array_is_list(array $array): bool",
    description:
      "Returns true when an array has consecutive integer keys starting at zero. Empty arrays are lists. Associative keys or gaps in numeric keys make an array stop being a list.",
    example: `let $empty = []
let $numbers = [1, 2, 3]

echo array_is_list($empty) . "\\n"
echo array_is_list($numbers) . "\\n"`,
  },
  {
    name: "function_exists",
    signature: "function_exists(string $function): bool",
    description:
      "Returns true when a name resolves to a function. Language constructs are not functions, so names such as echo and include_once return false.",
    example: `let $callable = "strlen"
let $construct = "echo"

echo function_exists($callable) . "\\n"
echo function_exists($construct) . "\\n"`,
  },
];

function headingId(heading: string) {
  return heading.toLowerCase().replaceAll(" ", "-");
}

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

function DocsShell({ category, title, headings, children }: DocsShellProps) {
  const location = useLocation();
  const navigation: DocsNavGroup[] = [
    {
      title: "Getting Started",
      links: [
        { label: "Installation", to: "/docs" },
        { label: "Quickstart", to: "/docs/quickstart", disabled: true },
        { label: "Source Modes", to: "/docs/source-modes" },
      ],
    },
    {
      title: "Language",
      links: [
        { label: "PHP Built-ins", to: "/docs/php-built-ins" },
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
        { label: "Source Builds", to: "/docs/source-builds" },
      ],
    },
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
                            location.pathname === link.to
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
          <p className="text-sm font-semibold text-slate-500">{category}</p>
          <h1 className="mt-6 text-5xl font-semibold tracking-normal text-slate-950">{title}</h1>
          {children}
        </article>

        <aside className="hidden xl:block">
          <nav aria-label="On this page" className="sticky top-32 border-l border-slate-200 pl-6">
            <h2 className="text-sm font-semibold text-slate-950">On this page</h2>
            <ul className="mt-5 space-y-3">
              {headings.map((heading) => (
                <li key={heading}>
                  <a
                    className="text-sm leading-6 text-slate-500 transition hover:text-slate-950"
                    href={`#${headingId(heading)}`}
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

function DocsPage() {
  return (
    <DocsShell
      category="Getting Started"
      headings={[
        "Meet Echo",
        "Installation",
        "Run a Program",
        "Compile a Program",
        "Project Status",
      ]}
      title="Installation"
    >
      <section id="meet-echo" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Meet Echo</h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Echo is a Rust implementation of a PHP superset. Existing PHP should stay familiar, while
          Echo adds compiler tooling, native concurrency, parallel execution, and a path toward
          compiled binaries with predictable performance gains.
        </p>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          The command line entrypoint is <code className="font-mono text-slate-950">xo</code>. Echo
          is early-stage software, so unsupported PHP behavior should fail explicitly rather than
          silently approximate semantics.
        </p>
      </section>

      <section id="installation" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Installation</h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Install the <code className="font-mono text-slate-950">xo</code> command and keep it on
          your path. The public installer flow is still being designed, so current releases are
          source-built by contributors.
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
          Use <code className="font-mono text-slate-950">xo run</code> to execute an Echo-compatible
          PHP file directly from the command line.
        </p>
        <pre className="mt-8 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
          <code>{`xo run examples/hello.php`}</code>
        </pre>
      </section>

      <section id="compile-a-program" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Compile a Program</h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Use <code className="font-mono text-slate-950">xo build</code> to compile a supported
          program into a native binary. The current backend lowers through LLVM IR and links through
          the project build path while Echo's native toolchain matures.
        </p>
        <pre className="mt-8 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
          <code>{`xo build examples/hello.php -o /tmp/hello
/tmp/hello`}</code>
        </pre>
      </section>

      <section id="project-status" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Project Status</h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Echo currently supports a small but growing PHP-compatible slice across parsing, AST
          generation, LLVM IR codegen, runtime behavior, and CLI execution. The docs should make
          that boundary visible as the language grows.
        </p>
      </section>
    </DocsShell>
  );
}

function SourceModesPage() {
  return (
    <DocsShell category="Getting Started" headings={["Source Modes"]} title="Source Modes">
      <section id="source-modes" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Source Modes</h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Files ending in <code className="font-mono text-slate-950">.php</code> use Echo superset
          mode by default. Files ending in <code className="font-mono text-slate-950">.echo</code>{" "}
          or <code className="font-mono text-slate-950">.xo</code> use strict mode by default.
        </p>
      </section>
    </DocsShell>
  );
}

function PhpBuiltinsPage() {
  return (
    <DocsShell
      category="Language"
      headings={builtinDocs.map((builtin) => builtin.name)}
      title="PHP Built-ins"
    >
      <p className="mt-6 text-lg leading-8 text-slate-600">
        PHP built-ins keep familiar names and signatures. These pages focus on what each function
        does, the shape of its inputs and output, and small examples that can be pasted into an
        Echo-compatible file.
      </p>

      <div className="mt-10 divide-y divide-slate-200 border-y border-slate-200">
        {builtinDocs.map((builtin) => (
          <section key={builtin.name} className="py-8">
            <h2
              className="font-mono text-2xl font-semibold text-slate-950"
              id={headingId(builtin.name)}
            >
              {builtin.name}
            </h2>
            <p className="mt-3 font-mono text-sm text-slate-500">{builtin.signature}</p>
            <p className="mt-7 text-lg leading-8 text-slate-600">{builtin.description}</p>

            <pre className="mt-7 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
              <code>{builtin.example}</code>
            </pre>
          </section>
        ))}
      </div>
    </DocsShell>
  );
}

function SourceBuildsPage() {
  return (
    <DocsShell category="Tooling" headings={["Source Builds"]} title="Source Builds">
      <section id="source-builds" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Source Builds</h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Contributors can build the current command line from source. Full workspace builds require
          Rust, LLVM 22, clang, and PHP for compatibility fixture generation.
        </p>
        <pre className="mt-8 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
          <code>{`git clone https://github.com/modoterra/echo.git
cd echo
cargo build -p xo
cargo test --workspace
cargo run -p xo -- run examples/hello.php`}</code>
        </pre>
      </section>
    </DocsShell>
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

const sourceModesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/source-modes",
  component: SourceModesPage,
});

const phpBuiltinsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/php-built-ins",
  component: PhpBuiltinsPage,
});

const sourceBuildsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/source-builds",
  component: SourceBuildsPage,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  docsRoute,
  sourceModesRoute,
  phpBuiltinsRoute,
  sourceBuildsRoute,
]);

export const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
