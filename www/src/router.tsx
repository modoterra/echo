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
    href?: string;
  }>;
};

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
        { label: "Installation", href: "#installation" },
        { label: "Project Status", href: "#project-status" },
        { label: "Run a Program", href: "#run-a-program" },
        { label: "Source Modes", href: "#source-modes" },
      ],
    },
    {
      title: "Language",
      links: [{ label: "PHP Compatibility" }, { label: "Strict Mode" }, { label: "Imports" }],
    },
    {
      title: "Tooling",
      links: [{ label: "Command Line" }, { label: "Language Server" }, { label: "Testing" }],
    },
  ];

  const headings = ["Meet Echo", "Installation", "Project Status", "Run a Program", "Source Modes"];

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
                      {link.href ? (
                        <a
                          className="text-sm leading-6 text-slate-500 transition hover:text-slate-950"
                          href={link.href}
                        >
                          {link.label}
                        </a>
                      ) : (
                        <span className="text-sm leading-6 text-slate-300">{link.label}</span>
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
              Build Echo from the repository while the language and tooling are still moving.
              Development builds require Rust, LLVM 22, clang, PHP, Node.js, and npm.
            </p>
            <pre className="mt-8 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
              <code>{`git clone https://github.com/modoterra/echo.git
cd echo
cargo test --workspace
cd www
npm install
npm run build`}</code>
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

          <section id="run-a-program" className="mt-16 scroll-mt-28">
            <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Run a Program</h2>
            <p className="mt-6 text-lg leading-8 text-slate-600">
              Use <code className="font-mono text-slate-950">xo</code> to inspect, run, or build an
              Echo-compatible PHP file.
            </p>
            <pre className="mt-8 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
              <code>{`cargo run -p xo -- ast examples/hello.php
cargo run -p xo -- ir examples/hello.php
cargo run -p xo -- run examples/hello.php
cargo run -p xo -- build examples/hello.php -o /tmp/hello`}</code>
            </pre>
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
