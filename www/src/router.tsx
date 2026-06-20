import { createRootRoute, createRoute, createRouter, Link, Outlet } from "@tanstack/react-router";
import { HomePage } from "./app";

function RootLayout() {
  return (
    <>
      <nav
        aria-label="Primary navigation"
        className="absolute left-1/2 top-8 z-20 flex w-full max-w-[624px] -translate-x-1/2 items-center justify-start gap-8 px-6 text-sm font-semibold text-slate-500 sm:top-12 sm:px-0"
      >
        <Link className="transition hover:text-slate-950" to="/">
          Home
        </Link>
        <Link className="transition hover:text-slate-950" to="/docs">
          Docs
        </Link>
      </nav>
      <Outlet />
    </>
  );
}

function DocsPage() {
  return (
    <main className="min-h-screen bg-white px-6 pt-32 text-slate-950">
      <section className="mx-auto w-full max-w-[624px]">
        <p className="text-sm font-semibold text-slate-500">Documentation</p>
        <h1 className="mt-6 text-4xl font-semibold tracking-normal text-slate-950">Docs</h1>
        <p className="mt-6 max-w-xl text-base leading-7 text-slate-600">
          Echo documentation will live here.
        </p>
      </section>
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
