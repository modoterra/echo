import {
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Outlet,
  useLocation,
  useNavigate,
} from "@tanstack/react-router";
import { RiCloseLine, RiMenuLine } from "@remixicon/react";
import { AnimatePresence, motion } from "motion/react";
import {
  createContext,
  useContext,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { HomePage } from "./app";
import { CtaLink } from "./components/cta-link";
import { DocsSearch } from "./components/docs-search";
import { EchoCode } from "./components/echo-code";
import { Logo } from "./components/logo";
import { InstallPage } from "./install";
import {
  docsPageByPath,
  docsPages,
  flattenNavPaths,
  headingId,
  navigationForPath,
  type DocsBlock,
  type DocsNavGroup,
  type DocsNavLink,
  type DocsPage,
  type DocsTextPart,
} from "./docs/content";

type FooterLink = {
  label: string;
  href: string;
  disabled?: boolean;
};

type FooterLinkGroup = {
  title: string;
  links: FooterLink[];
};

const footerLinkGroups: FooterLinkGroup[] = [
  {
    title: "Learn",
    links: [
      { label: "Install", href: "/install" },
      { label: "First program", href: "/docs/first-program" },
      { label: "Reference", href: "/docs" },
      { label: "Book", href: "/book" },
      { label: "Echo 2026", href: "/e26" },
    ],
  },
  {
    title: "Community",
    links: [
      {
        label: "GitHub",
        href: "https://github.com/modoterra/echo",
      },
      { label: "Discord", href: "#", disabled: true },
    ],
  },
  {
    title: "About",
    links: [
      {
        label: "Modoterra",
        href: "https://modoterra.xyz",
      },
    ],
  },
];

type DocsShellProps = {
  category: string;
  title: string;
  headings: string[];
  children: ReactNode;
};

type DocsPageMeta = Omit<DocsShellProps, "children">;

type DocsLayoutContextValue = {
  setMeta: (meta: DocsPageMeta) => void;
};

const defaultDocsPageMeta: DocsPageMeta = {
  category: docsPages[0].category,
  headings: docsPages[0].sections.map((section) => section.title),
  title: docsPages[0].title,
};

const DocsLayoutContext = createContext<DocsLayoutContextValue | null>(null);

function docsPage(path: string) {
  const page = docsPageByPath.get(path);
  if (!page) {
    throw new Error(`Missing docs page: ${path}`);
  }
  return page;
}

function scrollElementIntoContainerView(
  container: HTMLElement,
  element: HTMLElement,
  behavior: ScrollBehavior,
) {
  const containerRect = container.getBoundingClientRect();
  const elementRect = element.getBoundingClientRect();
  const topOverflow = elementRect.top - containerRect.top;
  const bottomOverflow = elementRect.bottom - containerRect.bottom;

  if (topOverflow < 0) {
    container.scrollBy({ behavior, top: topOverflow - 8 });
    return;
  }

  if (bottomOverflow > 0) {
    container.scrollBy({ behavior, top: bottomOverflow + 8 });
  }
}

function Topbar() {
  return (
    <header className="fixed inset-x-0 top-0 z-30 border-b border-slate-200/70 bg-white/85 px-6 shadow-2xs backdrop-blur">
      <div className="mx-auto grid h-20 w-full max-w-7xl grid-cols-[auto_1fr_auto] items-center gap-4 sm:gap-6 lg:grid-cols-[220px_minmax(0,720px)_minmax(220px,auto)] lg:gap-12">
        <Link
          aria-label="Echo home"
          className="block w-16 opacity-90 transition hover:opacity-100 lg:w-20"
          to="/"
        >
          <img alt="Echo" className="h-8 w-full" src="/logo.svg" />
        </Link>
        <nav
          aria-label="Primary navigation"
          className="flex translate-x-0.5 items-center justify-start gap-4 text-sm font-semibold text-slate-500 sm:gap-8 lg:translate-x-0.5"
        >
          <Link className="transition hover:text-slate-950" to="/">
            Home
          </Link>
          {/* Path union is incomplete when child routes are built as arrays. */}
          <Link className="transition hover:text-slate-950" to={"/docs" as "/"}>
            Docs
          </Link>
          <Link className="transition hover:text-slate-950" to={"/book" as "/"}>
            Book
          </Link>
          <Link className="hidden transition hover:text-slate-950 sm:inline" to={"/e26" as "/"}>
            Echo 2026
          </Link>
        </nav>
        <div className="flex items-center justify-end gap-2 sm:gap-3">
          <span className="hidden xl:inline-flex">
            <DocsSearch />
          </span>
          <span className="inline-flex xl:hidden">
            <DocsSearch iconOnly />
          </span>
          <CtaLink compact to="/install">
            Install
          </CtaLink>
        </div>
      </div>
    </header>
  );
}

function SiteFooter() {
  return (
    <footer className="overflow-hidden border-t border-slate-200 bg-white px-6 pt-24 text-slate-600">
      <div className="mx-auto grid w-full max-w-7xl gap-14 lg:grid-cols-[minmax(0,360px)_1fr]">
        <section>
          <p className="max-w-sm text-xl font-semibold leading-8 text-slate-950">Echo</p>
          <p className="mt-5 max-w-sm text-sm leading-6 text-slate-500">
            A compiled language with leaders instead of keywords. Write clear programs; ship native
            binaries with xo.
          </p>
          <p className="mt-10 text-sm text-slate-400">© 2026 Modoterra Corporation</p>
        </section>

        <nav aria-label="Footer navigation" className="grid gap-10 sm:grid-cols-2 lg:grid-cols-4">
          {footerLinkGroups.map((group) => (
            <section key={group.title}>
              <h2 className="text-sm font-semibold text-slate-950">{group.title}</h2>
              <ul className="mt-6 flex flex-col gap-4">
                {group.links.map((link) => (
                  <li key={link.label}>
                    {link.disabled ? (
                      <span className="text-sm text-slate-300">{link.label}</span>
                    ) : link.href.startsWith("http") ? (
                      <a
                        className="text-sm text-slate-500 transition hover:text-slate-950"
                        href={link.href}
                        rel="noreferrer"
                        target="_blank"
                      >
                        {link.label}
                      </a>
                    ) : (
                      <Link
                        className="text-sm text-slate-500 transition hover:text-slate-950"
                        to={link.href}
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
      </div>

      <div className="mx-auto mt-40 flex w-full max-w-7xl justify-end pb-10">
        <div className="w-full max-w-md opacity-90">
          <Logo />
        </div>
      </div>
    </footer>
  );
}

function RootLayout() {
  return (
    <>
      <Topbar />
      <Outlet />
      <SiteFooter />
    </>
  );
}

function NotFoundPage() {
  return (
    <main className="bg-white px-6 py-28 text-slate-950">
      <section className="mx-auto max-w-3xl">
        <p className="text-sm font-semibold uppercase tracking-[0.18em] text-slate-500">404</p>
        <h1 className="mt-5 text-4xl font-semibold leading-tight sm:text-5xl">Page not found</h1>
        <p className="mt-5 max-w-2xl text-lg leading-8 text-slate-600">
          The requested page does not exist or has moved.
        </p>
        <div className="mt-9 flex flex-wrap gap-3">
          <Link
            className="inline-flex items-center justify-center rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white transition hover:bg-slate-800"
            to={"/book" as "/"}
          >
            Open the Book
          </Link>
          <Link
            className="inline-flex items-center justify-center rounded-md border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-700 transition hover:border-slate-400 hover:text-slate-950"
            to="/"
          >
            Back home
          </Link>
        </div>
      </section>
    </main>
  );
}

function DocsNavLinkItem({
  link,
  onNavigate,
  pathname,
  itemRef,
}: {
  link: DocsNavLink;
  onNavigate?: () => void;
  pathname: string;
  itemRef?: (element: HTMLLIElement | null) => void;
}) {
  const isActive = pathname === link.to;
  const hasActiveChild = link.children?.some((child) => pathname === child.to);
  const activeChildIndex = link.children?.findIndex((child) => pathname === child.to) ?? -1;
  const shouldShowChildren = Boolean(link.children && (isActive || hasActiveChild));
  const childRailRef = useRef<HTMLDivElement | null>(null);
  const childItemRefs = useRef<Record<string, HTMLLIElement | null>>({});
  const [childTrainY, setChildTrainY] = useState(0);
  const textClass = link.disabled
    ? "text-sm leading-6 text-slate-300"
    : isActive
      ? "text-sm font-semibold leading-6 text-slate-950"
      : "text-sm leading-6 text-slate-500 transition hover:text-slate-950";

  useLayoutEffect(() => {
    if (!shouldShowChildren || activeChildIndex < 0) {
      setChildTrainY(0);
      return;
    }

    let animationFrame = 0;

    function updateChildTrainPosition() {
      const rail = childRailRef.current;
      const activeChild = link.children?.[activeChildIndex];
      const item = activeChild ? childItemRefs.current[activeChild.to] : null;

      if (!rail || !item) {
        setChildTrainY(0);
        return;
      }

      const railRect = rail.getBoundingClientRect();
      const itemRect = item.getBoundingClientRect();
      setChildTrainY(itemRect.top - railRect.top + itemRect.height / 2 - 9);
    }

    function scheduleUpdate() {
      window.cancelAnimationFrame(animationFrame);
      animationFrame = window.requestAnimationFrame(updateChildTrainPosition);
    }

    scheduleUpdate();
    window.addEventListener("resize", scheduleUpdate);

    return () => {
      window.cancelAnimationFrame(animationFrame);
      window.removeEventListener("resize", scheduleUpdate);
    };
  }, [activeChildIndex, link.children, shouldShowChildren]);

  return (
    <li ref={itemRef}>
      {link.disabled ? (
        <span className={textClass}>{link.label}</span>
      ) : (
        <Link className={textClass} onClick={onNavigate} to={link.to}>
          {link.label}
        </Link>
      )}
      <AnimatePresence initial={false}>
        {shouldShowChildren ? (
          <motion.div
            animate={{ height: "auto", opacity: 1 }}
            className="overflow-hidden"
            exit={{ height: 0, opacity: 0 }}
            initial={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2, ease: "easeOut" }}
          >
            <div className="relative mt-3 pl-3" ref={childRailRef}>
              <span
                aria-hidden="true"
                className="absolute bottom-0 left-0 top-0 w-[3px] bg-slate-200"
              />
              {activeChildIndex >= 0 ? (
                <span
                  aria-hidden="true"
                  className="docs-primary-nav-train docs-logo-gradient-rail absolute left-0 top-[3px] h-[18px] w-[3px] rounded-full transition-transform duration-200 ease-out"
                  style={{ transform: `translateY(${childTrainY}px)` }}
                />
              ) : null}
              <ul className="space-y-3">
                {link.children?.map((child) => (
                  <DocsNavLinkItem
                    key={child.label}
                    link={child}
                    onNavigate={onNavigate}
                    pathname={pathname}
                    itemRef={(element) => {
                      childItemRefs.current[child.to] = element;
                    }}
                  />
                ))}
              </ul>
            </div>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </li>
  );
}

function DocsNavigationList({
  navigation,
  onNavigate,
  pathname,
}: {
  navigation: DocsNavGroup[];
  onNavigate?: () => void;
  pathname: string;
}) {
  return (
    <div className="space-y-10">
      {navigation.map((group) => (
        <section key={group.title}>
          <h2 className="text-sm font-semibold text-slate-950">{group.title}</h2>
          <ul className="mt-5 space-y-3">
            {group.links.map((link) => (
              <DocsNavLinkItem
                key={link.label}
                link={link}
                onNavigate={onNavigate}
                pathname={pathname}
              />
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

function DocsShell({ category, title, headings, children }: DocsShellProps) {
  const docsLayout = useContext(DocsLayoutContext);

  useLayoutEffect(() => {
    docsLayout?.setMeta({ category, headings, title });
  }, [category, docsLayout, headings, title]);

  return <>{children}</>;
}

function DocsLayout() {
  const location = useLocation();
  const navigation = useMemo(() => navigationForPath(location.pathname), [location.pathname]);
  const [meta, setMeta] = useState<DocsPageMeta>(defaultDocsPageMeta);
  const docsLayoutContext = useMemo(() => ({ setMeta }), []);
  const { category, headings, title } = meta;
  const [activeHeading, setActiveHeading] = useState(headings[0] ?? "");
  const [isMobileNavOpen, setIsMobileNavOpen] = useState(false);
  const onThisPageViewportRef = useRef<HTMLDivElement | null>(null);
  const onThisPageRailRef = useRef<HTMLDivElement | null>(null);
  const onThisPageItemRefs = useRef<Record<string, HTMLLIElement | null>>({});
  const [onThisPageTrainY, setOnThisPageTrainY] = useState(0);

  useEffect(() => {
    setIsMobileNavOpen(false);
  }, [location.pathname]);

  useEffect(() => {
    if (!isMobileNavOpen) {
      return;
    }

    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";

    return () => {
      document.body.style.overflow = previousOverflow;
    };
  }, [isMobileNavOpen]);

  useEffect(() => {
    let animationFrame = 0;

    function updateActiveHeading() {
      const nextActiveHeading =
        headings.findLast((heading) => {
          const element = document.getElementById(headingId(heading));
          return element ? element.getBoundingClientRect().top <= 160 : false;
        }) ??
        headings.find((heading) => document.getElementById(headingId(heading))) ??
        headings[0] ??
        "";

      setActiveHeading(nextActiveHeading);
    }

    function scheduleUpdate() {
      window.cancelAnimationFrame(animationFrame);
      animationFrame = window.requestAnimationFrame(updateActiveHeading);
    }

    setActiveHeading(headings[0] ?? "");
    scheduleUpdate();
    window.addEventListener("scroll", scheduleUpdate, { passive: true });
    window.addEventListener("resize", scheduleUpdate);

    return () => {
      window.cancelAnimationFrame(animationFrame);
      window.removeEventListener("scroll", scheduleUpdate);
      window.removeEventListener("resize", scheduleUpdate);
    };
  }, [headings]);

  useLayoutEffect(() => {
    let animationFrame = 0;

    function updateTrainPosition() {
      const rail = onThisPageRailRef.current;
      const item = onThisPageItemRefs.current[activeHeading];

      if (!rail || !item) {
        setOnThisPageTrainY(0);
        return;
      }

      const railRect = rail.getBoundingClientRect();
      const itemRect = item.getBoundingClientRect();
      setOnThisPageTrainY(itemRect.top - railRect.top + itemRect.height / 2 - 9);
    }

    function scheduleUpdate() {
      window.cancelAnimationFrame(animationFrame);
      animationFrame = window.requestAnimationFrame(updateTrainPosition);
    }

    scheduleUpdate();
    window.addEventListener("resize", scheduleUpdate);

    return () => {
      window.cancelAnimationFrame(animationFrame);
      window.removeEventListener("resize", scheduleUpdate);
    };
  }, [activeHeading, headings]);

  useLayoutEffect(() => {
    const container = onThisPageViewportRef.current;
    const item = onThisPageItemRefs.current[activeHeading];

    if (!container || !item) {
      return;
    }

    scrollElementIntoContainerView(container, item, "smooth");
  }, [activeHeading]);

  function scrollToHeading(heading: string) {
    const id = headingId(heading);
    const element = document.getElementById(id);

    if (!element) {
      return;
    }

    window.history.pushState(null, "", `#${id}`);
    window.scrollTo({
      behavior: "smooth",
      top: element.getBoundingClientRect().top + window.scrollY - 112,
    });
  }

  return (
    <main className="min-h-screen bg-white px-6 pb-24 pt-36 text-slate-950 lg:pt-32">
      <div className="fixed inset-x-0 top-20 z-20 border-b border-slate-200/70 bg-white/95 px-6 backdrop-blur lg:hidden">
        <div className="mx-auto flex h-14 w-full max-w-7xl items-center justify-between">
          <button
            aria-controls="mobile-docs-menu"
            aria-expanded={isMobileNavOpen}
            aria-label="Open documentation menu"
            className="inline-flex size-10 items-center justify-center rounded-md border border-slate-200 bg-white text-slate-500 transition hover:border-slate-300 hover:text-slate-950"
            onClick={() => setIsMobileNavOpen(true)}
            type="button"
          >
            <RiMenuLine size={18} />
          </button>
          <DocsSearch iconOnly onNavigate={() => setIsMobileNavOpen(false)} />
        </div>
      </div>

      <AnimatePresence>
        {isMobileNavOpen ? (
          <motion.div
            animate={{ opacity: 1 }}
            aria-label="Documentation menu"
            aria-modal="true"
            className="fixed inset-0 z-40 bg-white lg:hidden"
            exit={{ opacity: 0 }}
            id="mobile-docs-menu"
            initial={{ opacity: 0 }}
            role="dialog"
            transition={{ duration: 0.16, ease: "easeOut" }}
          >
            <div className="flex h-full flex-col">
              <div className="flex h-20 items-center justify-between border-b border-slate-200 px-6">
                <Link
                  aria-label="Echo home"
                  className="block w-16 opacity-90 transition hover:opacity-100"
                  to="/"
                >
                  <img alt="Echo" className="h-8 w-full" src="/logo.svg" />
                </Link>
                <button
                  aria-label="Close documentation menu"
                  className="inline-flex size-10 items-center justify-center rounded-md border border-slate-200 text-slate-500 transition hover:border-slate-300 hover:text-slate-950"
                  onClick={() => setIsMobileNavOpen(false)}
                  type="button"
                >
                  <RiCloseLine size={20} />
                </button>
              </div>
              <div className="border-b border-slate-100 px-6 py-5">
                <DocsSearch fullWidth onNavigate={() => setIsMobileNavOpen(false)} />
              </div>
              <nav
                aria-label="Documentation sections"
                className="scrollbar-nice flex-1 overflow-y-auto px-6 py-7"
              >
                <DocsNavigationList
                  navigation={navigation}
                  onNavigate={() => setIsMobileNavOpen(false)}
                  pathname={location.pathname}
                />
              </nav>
            </div>
          </motion.div>
        ) : null}
      </AnimatePresence>

      <div className="mx-auto grid w-full max-w-7xl grid-cols-1 gap-12 lg:grid-cols-[220px_minmax(0,720px)] xl:grid-cols-[220px_minmax(0,720px)_220px]">
        <aside className="hidden lg:block">
          <nav
            aria-label="Documentation sections"
            className="scrollbar-nice sticky top-32 max-h-[calc(100vh-12rem)] overflow-y-auto pr-2"
          >
            <DocsNavigationList navigation={navigation} pathname={location.pathname} />
          </nav>
        </aside>

        <DocsLayoutContext.Provider value={docsLayoutContext}>
          <article className="max-w-none">
            <p className="text-sm font-semibold text-slate-500">{category}</p>
            <h1 className="mt-6 text-5xl font-semibold tracking-normal text-slate-950">{title}</h1>
            <Outlet />
          </article>
        </DocsLayoutContext.Provider>

        <aside className="hidden xl:block">
          <nav aria-label="On this page" className="sticky top-32">
            <h2 className="text-xs font-semibold uppercase tracking-wide text-slate-400">
              On this page
            </h2>
            <div
              className="scrollbar-nice mt-5 max-h-[calc(100vh-12rem)] overflow-y-auto pr-2"
              ref={onThisPageViewportRef}
            >
              <div className="relative pl-6" ref={onThisPageRailRef}>
                <span
                  aria-hidden="true"
                  className="absolute bottom-0 left-0 top-0 w-px bg-slate-200"
                />
                <motion.span
                  aria-hidden="true"
                  animate={{ y: onThisPageTrainY }}
                  className="docs-on-this-page-train docs-logo-gradient-rail absolute left-[-1px] top-0 h-[18px] w-[3px] rounded-full"
                  transition={{ duration: 0.16, ease: "easeOut" }}
                />
                <ul className="docs-on-this-page-links space-y-3">
                  {headings.map((heading) => (
                    <li
                      key={heading}
                      ref={(element) => {
                        onThisPageItemRefs.current[heading] = element;
                      }}
                    >
                      <a
                        className={
                          activeHeading === heading
                            ? "text-sm font-semibold leading-6 text-slate-950 transition"
                            : "text-sm leading-6 text-slate-500 transition hover:text-slate-950"
                        }
                        href={`#${headingId(heading)}`}
                        onClick={(event) => {
                          if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) {
                            return;
                          }

                          event.preventDefault();
                          scrollToHeading(heading);
                        }}
                      >
                        {heading}
                      </a>
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </nav>
        </aside>
      </div>
    </main>
  );
}

function renderInlineText(parts: DocsTextPart[]) {
  return parts.map((part, index) =>
    typeof part === "string" ? (
      <span key={index}>{part}</span>
    ) : (
      <code
        key={index}
        className="rounded bg-slate-100 px-1.5 py-0.5 font-mono text-[0.9em] text-slate-800"
      >
        {part.code}
      </code>
    ),
  );
}

function renderBlock(block: DocsBlock, key: number) {
  if (block.kind === "paragraph") {
    return (
      <p key={key} className="mt-5 text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
        {renderInlineText(block.text)}
      </p>
    );
  }

  return <EchoCode key={key} code={block.code} language={block.language ?? "echo"} />;
}

function DocsContentPage({ page }: { page: DocsPage }) {
  const location = useLocation();
  const navigate = useNavigate();
  const sectionPaths = useMemo(
    () => flattenNavPaths(navigationForPath(location.pathname)),
    [location.pathname],
  );
  const currentIndex = sectionPaths.indexOf(location.pathname);
  const previousPath = currentIndex > 0 ? sectionPaths[currentIndex - 1] : null;
  const nextPath =
    currentIndex >= 0 && currentIndex + 1 < sectionPaths.length
      ? sectionPaths[currentIndex + 1]
      : null;

  useEffect(() => {
    if (!previousPath && !nextPath) {
      return;
    }

    function isTypingTarget(target: EventTarget | null) {
      if (!(target instanceof HTMLElement)) {
        return false;
      }

      const tagName = target.tagName;
      return (
        target.isContentEditable ||
        tagName === "INPUT" ||
        tagName === "TEXTAREA" ||
        tagName === "SELECT"
      );
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) {
        return;
      }

      if (isTypingTarget(event.target)) {
        return;
      }

      if (event.key === "j" && previousPath) {
        event.preventDefault();
        navigate({ to: previousPath });
      }

      if (event.key === "k" && nextPath) {
        event.preventDefault();
        navigate({ to: nextPath });
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [navigate, nextPath, previousPath]);

  return (
    <DocsShell
      category={page.category}
      headings={page.sections.map((section) => section.title)}
      title={page.title}
    >
      <p className="mt-6 text-lg leading-8 text-slate-600">{page.summary}</p>
      {page.sections.map((section) => (
        <section className="mt-16 scroll-mt-28" id={headingId(section.title)} key={section.title}>
          <h2 className="text-3xl font-semibold tracking-normal text-slate-950">{section.title}</h2>
          {section.blocks.map((block, index) => renderBlock(block, index))}
        </section>
      ))}
      {previousPath || nextPath ? (
        <div className="mt-16 flex w-full items-center justify-between">
          {previousPath ? (
            <Link
              className="inline-flex items-center justify-center rounded-md border border-slate-200 bg-slate-50 px-3 py-2 font-mono text-sm font-semibold tracking-wide text-slate-700 transition hover:border-slate-300 hover:bg-white hover:text-slate-950 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-300"
              to={previousPath}
            >
              Previous
            </Link>
          ) : (
            <span />
          )}
          {nextPath ? (
            <Link
              className="inline-flex items-center justify-center rounded-md border border-slate-200 bg-slate-50 px-3 py-2 font-mono text-sm font-semibold tracking-wide text-slate-700 transition hover:border-slate-300 hover:bg-white hover:text-slate-950 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-300"
              to={nextPath}
            >
              Next
            </Link>
          ) : null}
        </div>
      ) : null}
    </DocsShell>
  );
}

const rootRoute = createRootRoute({
  component: RootLayout,
  notFoundComponent: NotFoundPage,
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: HomePage,
});

const installRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/install",
  component: InstallPage,
});

// ── /docs (all paths from docsPages under /docs) ─────────────────────
const docsLayoutRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs",
  component: DocsLayout,
});
function docsChild(path: string) {
  const full = path === "/" ? "/docs" : `/docs/${path}`;
  return createRoute({
    getParentRoute: () => docsLayoutRoute,
    path,
    component: () => <DocsContentPage page={docsPage(full)} />,
  });
}
const docsChildren = [
  docsChild("/"),
  docsChild("first-program"),
  docsChild("project"),
  docsChild("leaders"),
  docsChild("binds"),
  docsChild("values"),
  docsChild("collections"),
  docsChild("control"),
  docsChild("result-option"),
  docsChild("strings"),
  docsChild("modules"),
  docsChild("structs"),
  docsChild("tasks"),
  docsChild("names"),
  docsChild("std"),
  docsChild("std/io-strings"),
  docsChild("std/tcp"),
  docsChild("std/udp"),
  docsChild("std/http"),
  docsChild("guides/packages"),
  docsChild("guides/diagnostics"),
  docsChild("guides/repl"),
  docsChild("guides/cookbook"),
  docsChild("toolchain"),
  docsChild("toolchain/commands"),
  docsChild("toolchain/examples"),
];

// ── /book ────────────────────────────────────────────────────────────
const bookLayoutRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/book",
  component: DocsLayout,
});
function bookChild(path: string) {
  const full = path === "/" ? "/book" : `/book/${path}`;
  return createRoute({
    getParentRoute: () => bookLayoutRoute,
    path,
    component: () => <DocsContentPage page={docsPage(full)} />,
  });
}
const bookChildren = [
  bookChild("/"),
  bookChild("leaders"),
  bookChild("binds"),
  bookChild("values"),
  bookChild("collections"),
  bookChild("control"),
  bookChild("result-option"),
  bookChild("strings"),
  bookChild("modules"),
  bookChild("structs"),
  bookChild("tasks"),
  bookChild("names"),
];

// ── /e26 ─────────────────────────────────────────────────────────────
const e26LayoutRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/e26",
  component: DocsLayout,
});
function e26Child(path: string) {
  const full = path === "/" ? "/e26" : `/e26/${path}`;
  return createRoute({
    getParentRoute: () => e26LayoutRoute,
    path,
    component: () => <DocsContentPage page={docsPage(full)} />,
  });
}
const e26Children = [e26Child("/"), e26Child("run"), e26Child("layout"), e26Child("protocol")];

const routeTree = rootRoute.addChildren([
  indexRoute,
  installRoute,
  docsLayoutRoute.addChildren(docsChildren),
  bookLayoutRoute.addChildren(bookChildren),
  e26LayoutRoute.addChildren(e26Children),
]);

export const router = createRouter({
  routeTree,
  defaultPreload: "intent",
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
