import {
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Outlet,
  useLocation,
} from "@tanstack/react-router";
import { RiCheckLine, RiFileCopyLine, RiSearchLine } from "@remixicon/react";
import { layout, prepare } from "@chenglou/pretext";
import { AnimatePresence, motion } from "motion/react";
import ShikiHighlighter, {
  createHighlighterCore,
  createJavaScriptRegexEngine,
} from "react-shiki/core";
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
import {
  builtinExample,
  builtinExampleNote,
  builtinFamilies,
  builtinFamilyBySlug,
  docsNavigation,
  docsPages,
  headingId,
  type BuiltinDoc,
  type BuiltinFamily,
  type DocsBlock,
  type DocsNavLink,
  type DocsPage as DocsPageData,
  type DocsTextPart,
} from "./docs/content";
import {
  cosineSimilarity,
  loadDocsMiniSearch,
  type DocsSearchAsset,
  type DocsSearchRecord,
  type DocsSemanticAsset,
} from "./docs/search";

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

type PhpHighlighter = Awaited<ReturnType<typeof createHighlighterCore>>;

let phpHighlighterPromise: Promise<PhpHighlighter> | null = null;

const codeSnippetFont = '14px "Geist Mono"';
const codeSnippetLineHeight = 28;
const codeSnippetBlockPadding = 48;
const codeSnippetSkeletonMinDelay = 120;
const codeSnippetSkeletonMaxDelay = 280;
const codeSnippetLoadRootMargin = "0px";

function loadPhpHighlighter() {
  phpHighlighterPromise ??= Promise.all([
    import("@shikijs/langs/php"),
    import("@shikijs/themes/github-dark"),
  ]).then(([php, githubDark]) =>
    createHighlighterCore({
      engine: createJavaScriptRegexEngine({ forgiving: true }),
      langs: [php.default],
      themes: [githubDark.default],
    }),
  );

  return phpHighlighterPromise;
}

function randomCodeSnippetSkeletonDelay() {
  const range = codeSnippetSkeletonMaxDelay - codeSnippetSkeletonMinDelay;

  return codeSnippetSkeletonMinDelay + Math.round(Math.random() * range);
}

type DocsSearchResult = Pick<
  DocsSearchRecord,
  "id" | "path" | "title" | "category" | "kind" | "excerpt"
> & {
  score: number;
  semanticScore?: number;
};

function DocsSearch() {
  const [isOpen, setIsOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [asset, setAsset] = useState<DocsSearchAsset | null>(null);
  const [semanticAsset, setSemanticAsset] = useState<DocsSemanticAsset | null>(null);
  const [queryEmbedding, setQueryEmbedding] = useState<number[] | null>(null);
  const [isLoadingIndex, setIsLoadingIndex] = useState(false);
  const [isLoadingModel, setIsLoadingModel] = useState(false);
  const [semanticUnavailable, setSemanticUnavailable] = useState(false);
  const searchInputRef = useRef<HTMLInputElement | null>(null);
  const miniSearch = useMemo(() => (asset ? loadDocsMiniSearch(asset) : null), [asset]);
  const results = useMemo(() => {
    const trimmedQuery = query.trim();

    if (!miniSearch || !trimmedQuery) {
      return [];
    }

    const lexicalResults = miniSearch.search(trimmedQuery) as unknown as DocsSearchResult[];

    if (!semanticAsset || !queryEmbedding) {
      return lexicalResults.slice(0, 8);
    }

    const semanticById = new Map(
      semanticAsset.records.map((record) => [
        record.id,
        cosineSimilarity(queryEmbedding, record.embedding),
      ]),
    );

    return lexicalResults
      .map((result) => {
        const semanticScore = semanticById.get(result.id) ?? 0;

        return {
          ...result,
          score: result.score * 0.65 + semanticScore * 0.35,
          semanticScore,
        };
      })
      .sort((left, right) => right.score - left.score)
      .slice(0, 8);
  }, [miniSearch, query, queryEmbedding, semanticAsset]);

  useEffect(() => {
    if (!isOpen || asset) {
      return;
    }

    let active = true;
    setIsLoadingIndex(true);

    void fetch("/docs-search-index.json")
      .then((response) => {
        if (!response.ok) {
          throw new Error(`Search index failed to load: ${response.status}`);
        }

        return response.json() as Promise<DocsSearchAsset>;
      })
      .then((loadedAsset) => {
        if (active) {
          setAsset(loadedAsset);
        }
      })
      .finally(() => {
        if (active) {
          setIsLoadingIndex(false);
        }
      });

    return () => {
      active = false;
    };
  }, [asset, isOpen]);

  useEffect(() => {
    if (!isOpen) {
      return;
    }

    window.setTimeout(() => searchInputRef.current?.focus(), 0);
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen || semanticAsset || semanticUnavailable || isLoadingModel) {
      return;
    }

    let active = true;

    void fetch("/docs-semantic-index.json")
      .then((response) => {
        if (!response.ok) {
          throw new Error("Semantic index is not available");
        }

        return response.json() as Promise<DocsSemanticAsset>;
      })
      .then((loadedSemanticAsset) => {
        if (active) {
          setSemanticAsset(loadedSemanticAsset);
        }
      })
      .catch(() => {
        if (active) {
          setSemanticUnavailable(true);
        }
      });

    return () => {
      active = false;
    };
  }, [isLoadingModel, isOpen, semanticAsset, semanticUnavailable]);

  useEffect(() => {
    const trimmedQuery = query.trim();

    if (!semanticAsset || !trimmedQuery) {
      setQueryEmbedding(null);
      return;
    }

    let active = true;
    setIsLoadingModel(true);

    void embedSearchQuery(trimmedQuery)
      .then((embedding) => {
        if (active) {
          setQueryEmbedding(embedding);
        }
      })
      .finally(() => {
        if (active) {
          setIsLoadingModel(false);
        }
      });

    return () => {
      active = false;
    };
  }, [query, semanticAsset]);

  useEffect(() => {
    function closeOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setIsOpen(false);
      }
    }

    if (isOpen) {
      window.addEventListener("keydown", closeOnEscape);
    }

    return () => {
      window.removeEventListener("keydown", closeOnEscape);
    };
  }, [isOpen]);

  return (
    <div className="relative">
      <button
        aria-expanded={isOpen}
        aria-label="Search documentation"
        className="inline-flex h-9 items-center gap-2 rounded-md border border-slate-200 bg-white px-3 text-sm font-semibold text-slate-500 transition hover:border-slate-300 hover:text-slate-950"
        onClick={() => setIsOpen((open) => !open)}
        type="button"
      >
        <RiSearchLine size={16} />
        <span className="hidden sm:inline">Search</span>
      </button>
      <AnimatePresence>
        {isOpen ? (
          <motion.div
            animate={{ opacity: 1, y: 0 }}
            className="absolute right-0 top-12 z-40 w-[min(28rem,calc(100vw-3rem))] rounded-lg border border-slate-200 bg-white p-3 shadow-xl"
            exit={{ opacity: 0, y: -4 }}
            initial={{ opacity: 0, y: -4 }}
            transition={{ duration: 0.16, ease: "easeOut" }}
          >
            <input
              className="h-10 w-full rounded-md border border-slate-200 px-3 text-sm text-slate-950 outline-none transition placeholder:text-slate-400 focus:border-slate-400"
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search docs"
              ref={searchInputRef}
              value={query}
            />
            <div className="mt-3 min-h-6 text-xs font-semibold uppercase tracking-wide text-slate-400">
              {isLoadingIndex
                ? "Loading index..."
                : isLoadingModel
                  ? "Loading model..."
                  : semanticAsset
                    ? "Semantic search ready"
                    : "Keyword and fuzzy search"}
            </div>
            <div className="mt-2 max-h-96 overflow-auto scrollbar-thin scrollbar-nice">
              {query.trim() && !isLoadingIndex && results.length === 0 ? (
                <p className="px-2 py-6 text-sm text-slate-500">No results found.</p>
              ) : null}
              <ul className="space-y-1">
                {results.map((result) => (
                  <li key={result.id}>
                    <Link
                      className="block rounded-md px-2 py-2 transition hover:bg-slate-50"
                      onClick={() => setIsOpen(false)}
                      to={result.path}
                    >
                      <span className="block text-sm font-semibold text-slate-950">
                        {result.title}
                      </span>
                      <span className="mt-1 block text-xs font-semibold uppercase tracking-wide text-slate-400">
                        {result.category} · {result.kind}
                      </span>
                      <span className="mt-1 line-clamp-2 block text-sm leading-6 text-slate-500">
                        {result.excerpt}
                      </span>
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}

let queryEmbedderPromise: Promise<{
  (
    query: string,
    options: { pooling: "mean"; normalize: true },
  ): Promise<{
    data: ArrayLike<number>;
  }>;
}> | null = null;

async function embedSearchQuery(query: string) {
  queryEmbedderPromise ??= import("@huggingface/transformers").then(async ({ env, pipeline }) => {
    env.localModelPath = "/models/";
    env.allowLocalModels = true;
    env.allowRemoteModels = false;

    return pipeline("feature-extraction", "xmlml6v2", {
      dtype: "q8",
    }) as unknown as {
      (
        query: string,
        options: { pooling: "mean"; normalize: true },
      ): Promise<{
        data: ArrayLike<number>;
      }>;
    };
  });

  const embedder = await queryEmbedderPromise;
  const output = await embedder(query, { pooling: "mean", normalize: true });

  return Array.from(output.data);
}

function CodeSnippet({ children, className = "mt-8" }: { children: string; className?: string }) {
  const snippetRef = useRef<HTMLDivElement | null>(null);
  const [copied, setCopied] = useState(false);
  const [highlighter, setHighlighter] = useState<PhpHighlighter | null>(null);
  const [shouldLoadHighlighter, setShouldLoadHighlighter] = useState(false);
  const code = children.trim();
  const minHeight = useMemo(() => {
    const prepared = prepare(code, codeSnippetFont, { whiteSpace: "pre-wrap" });
    const measured = layout(prepared, 100_000, codeSnippetLineHeight);

    return measured.height + codeSnippetBlockPadding;
  }, [code]);

  useEffect(() => {
    const snippet = snippetRef.current;

    if (!snippet) {
      return;
    }

    if (!("IntersectionObserver" in window)) {
      setShouldLoadHighlighter(true);
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry?.isIntersecting) {
          setShouldLoadHighlighter(true);
          observer.disconnect();
        }
      },
      { rootMargin: codeSnippetLoadRootMargin },
    );

    observer.observe(snippet);

    return () => {
      observer.disconnect();
    };
  }, []);

  useEffect(() => {
    if (!shouldLoadHighlighter) {
      return;
    }

    let active = true;
    let delayTimeout: number | undefined;
    const delay = new Promise((resolve) => {
      delayTimeout = window.setTimeout(resolve, randomCodeSnippetSkeletonDelay());
    });

    void Promise.all([loadPhpHighlighter(), delay]).then(([loadedHighlighter]) => {
      if (active) {
        setHighlighter(loadedHighlighter);
      }
    });

    return () => {
      active = false;
      window.clearTimeout(delayTimeout);
    };
  }, [shouldLoadHighlighter]);

  async function copyCode() {
    await navigator.clipboard.writeText(code);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1400);
  }

  return (
    <div
      ref={snippetRef}
      className={`${className} group relative rounded-lg bg-[#101218] shadow-sm`}
      style={{ minHeight }}
    >
      <button
        aria-label={copied ? "Copied code" : "Copy code"}
        className="absolute right-3 top-3 z-10 inline-flex size-8 select-none items-center justify-center rounded-md text-slate-400 opacity-70 transition hover:bg-white/10 hover:text-slate-100 hover:opacity-100 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-400"
        onClick={copyCode}
        title={copied ? "Copied" : "Copy"}
        type="button"
      >
        {copied ? <RiCheckLine size={18} /> : <RiFileCopyLine size={18} />}
      </button>
      <AnimatePresence initial={false} mode="wait">
        {highlighter ? (
          <motion.div
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            initial={{ opacity: 0 }}
            key="highlighted"
            transition={{ duration: 0.22, ease: "easeOut" }}
          >
            <ShikiHighlighter
              addDefaultStyles={false}
              className="docs-code-snippet overflow-x-auto rounded-lg p-6 pr-14 font-mono text-sm leading-7 scrollbar-thin scrollbar-nice-dark"
              highlighter={highlighter}
              language="php"
              showLanguage={false}
              showLineNumbers
              theme="github-dark"
            >
              {code}
            </ShikiHighlighter>
          </motion.div>
        ) : (
          <motion.div
            animate={{ opacity: 1 }}
            aria-label="Loading highlighted code"
            className="docs-code-skeleton p-6 pr-14"
            exit={{ opacity: 0 }}
            initial={{ opacity: 0 }}
            key="skeleton"
            transition={{ duration: 0.18, ease: "easeOut" }}
          >
            {code.split("\n").map((line, index) => (
              <div className="flex h-7 items-center gap-5" key={index}>
                <span className="h-3 w-5 shrink-0 rounded-full bg-slate-700/45" />
                <span
                  className="h-3 rounded-full bg-slate-700/55"
                  style={{
                    width: `${Math.min(92, Math.max(24, line.length * 1.8))}%`,
                  }}
                />
              </div>
            ))}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function Topbar() {
  return (
    <header className="fixed inset-x-0 top-0 z-30 border-b border-slate-200/70 bg-white/85 px-6 backdrop-blur shadow-2xs">
      <div className="mx-auto grid h-20 w-full max-w-7xl grid-cols-[auto_1fr] items-center gap-10 lg:grid-cols-[220px_minmax(0,720px)] lg:gap-12 xl:grid-cols-[220px_minmax(0,720px)_220px]">
        <Link
          aria-label="Echo home"
          className="block w-16 transition opacity-90 hover:opacity-100 lg:w-20"
          to="/"
        >
          <img alt="Echo" className="h-8 w-full" src="/logo.svg" />
        </Link>
        <nav
          aria-label="Primary navigation"
          className="flex translate-x-0.5 items-center justify-start gap-8 text-sm font-semibold text-slate-500 lg:translate-x-0.5"
        >
          <Link className="transition hover:text-slate-950" to="/">
            Home
          </Link>
          <Link className="transition hover:text-slate-950" to="/docs">
            Docs
          </Link>
        </nav>
        <div className="hidden justify-self-end xl:block">
          <DocsSearch />
        </div>
      </div>
    </header>
  );
}

function RootLayout() {
  return (
    <>
      <Topbar />
      <Outlet />
    </>
  );
}

function DocsNavLinkItem({ link, pathname }: { link: DocsNavLink; pathname: string }) {
  const isActive = pathname === link.to;
  const hasActiveChild = link.children?.some((child) => pathname === child.to);
  const activeChildIndex = link.children?.findIndex((child) => pathname === child.to) ?? -1;
  const shouldShowChildren = Boolean(link.children && (isActive || hasActiveChild));
  const textClass = link.disabled
    ? "text-sm leading-6 text-slate-300"
    : isActive
      ? "text-sm font-semibold leading-6 text-slate-950"
      : "text-sm leading-6 text-slate-500 transition hover:text-slate-950";

  return (
    <li>
      {link.disabled ? (
        <span className={textClass}>{link.label}</span>
      ) : (
        <Link className={textClass} to={link.to}>
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
            <div className="relative mt-3 pl-3">
              <span
                aria-hidden="true"
                className="absolute bottom-0 left-0 top-0 w-[3px] bg-slate-200"
              />
              {activeChildIndex >= 0 ? (
                <span
                  aria-hidden="true"
                  className="docs-primary-nav-train absolute left-0 top-[3px] h-[18px] w-[3px] rounded-full bg-orange-400 transition-transform duration-200 ease-out"
                  style={{
                    transform: `translateY(${activeChildIndex * 36}px)`,
                  }}
                />
              ) : null}
              <ul className="space-y-3">
                {link.children?.map((child) => (
                  <DocsNavLinkItem key={child.label} link={child} pathname={pathname} />
                ))}
              </ul>
            </div>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </li>
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
  const [meta, setMeta] = useState<DocsPageMeta>(defaultDocsPageMeta);
  const docsLayoutContext = useMemo(() => ({ setMeta }), []);
  const { category, headings, title } = meta;
  const [activeHeading, setActiveHeading] = useState(headings[0] ?? "");
  const onThisPageRailRef = useRef<HTMLDivElement | null>(null);
  const onThisPageItemRefs = useRef<Record<string, HTMLLIElement | null>>({});
  const [onThisPageTrainY, setOnThisPageTrainY] = useState(0);
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

  function scrollToHeading(heading: string) {
    const id = headingId(heading);
    const element = document.getElementById(id);

    if (!element) {
      return;
    }

    setActiveHeading(heading);
    window.history.pushState(null, "", `#${id}`);
    element.scrollIntoView({ behavior: "smooth", block: "start" });
  }

  return (
    <main className="min-h-screen bg-white px-6 pb-24 pt-32 text-slate-950">
      <div className="mx-auto grid w-full max-w-7xl grid-cols-1 gap-12 lg:grid-cols-[220px_minmax(0,720px)] xl:grid-cols-[220px_minmax(0,720px)_220px]">
        <aside className="hidden lg:block">
          <nav aria-label="Documentation sections" className="sticky top-32 space-y-10">
            {docsNavigation.map((group) => (
              <section key={group.title}>
                <h2 className="text-sm font-semibold text-slate-950">{group.title}</h2>
                <ul className="mt-5 space-y-3">
                  {group.links.map((link) => (
                    <DocsNavLinkItem key={link.label} link={link} pathname={location.pathname} />
                  ))}
                </ul>
              </section>
            ))}
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
            <div className="relative mt-5 pl-6" ref={onThisPageRailRef}>
              <span
                aria-hidden="true"
                className="absolute bottom-0 left-0 top-0 w-px bg-slate-200"
              />
              <motion.span
                aria-hidden="true"
                animate={{ y: onThisPageTrainY }}
                className="absolute left-[-1px] top-0 h-[18px] w-[3px] rounded-full bg-orange-400"
                transition={{ duration: 0.22, ease: "easeOut" }}
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
          </nav>
        </aside>
      </div>
    </main>
  );
}

function renderTextPart(part: DocsTextPart, index: number) {
  if (typeof part === "string") {
    return part;
  }

  return (
    <code className="font-mono text-slate-950" key={index}>
      {part.code}
    </code>
  );
}

function DocsBlockView({ block }: { block: DocsBlock }) {
  if (block.kind === "code") {
    return <CodeSnippet>{block.code}</CodeSnippet>;
  }

  return <p className="mt-6 text-lg leading-8 text-slate-600">{block.text.map(renderTextPart)}</p>;
}

function DocsContentPage({ page }: { page: DocsPageData }) {
  return (
    <DocsShell
      category={page.category}
      headings={page.sections.map((section) => section.title)}
      title={page.title}
    >
      {page.sections.map((section) => (
        <section className="mt-16 scroll-mt-28" id={headingId(section.title)} key={section.title}>
          <h2 className="text-3xl font-semibold tracking-normal text-slate-950">{section.title}</h2>
          {section.blocks.map((block, index) => (
            <DocsBlockView block={block} key={index} />
          ))}
        </section>
      ))}
    </DocsShell>
  );
}

function PhpBuiltinsPage() {
  return (
    <DocsShell
      category="Language"
      headings={builtinFamilies.map((family) => family.title)}
      title="PHP Built-ins"
    >
      <p className="mt-6 text-lg leading-8 text-slate-600">
        PHP built-ins keep familiar names and signatures. They are grouped by family so each page
        can stay focused: strings, arrays, types, math, filesystem, reflection, shell integration,
        output buffering, and core runtime helpers.
      </p>

      <div className="mt-10 grid gap-6 sm:grid-cols-2">
        {builtinFamilies.map((family) => (
          <section
            key={family.slug}
            className="border-t border-slate-200 pt-6"
            id={headingId(family.title)}
          >
            <h2 className="text-2xl font-semibold tracking-normal text-slate-950">
              {family.title}
            </h2>
            <p className="mt-4 text-base leading-7 text-slate-600">{family.description}</p>
            <a
              className="mt-5 inline-flex text-sm font-semibold text-slate-500 transition hover:text-slate-950"
              href={`/docs/php-built-ins/${family.slug}`}
            >
              {family.builtins.length} functions
            </a>
          </section>
        ))}
      </div>
    </DocsShell>
  );
}

function PhpBuiltinFamilyPage({ family }: { family: BuiltinFamily }) {
  return (
    <DocsShell
      category="PHP Built-ins"
      headings={family.builtins.map((builtin) => builtin.name)}
      title={family.title}
    >
      <p className="mt-6 text-lg leading-8 text-slate-600">{family.description}</p>

      <div className="mt-10 divide-y divide-slate-200 border-y border-slate-200">
        {family.builtins.map((builtin) => (
          <BuiltinReference key={builtin.name} builtin={builtin} />
        ))}
      </div>
    </DocsShell>
  );
}

function BuiltinReference({ builtin }: { builtin: BuiltinDoc }) {
  const example = builtinExample(builtin.name);
  const exampleNote = builtinExampleNote(builtin);

  return (
    <section className="py-8">
      <h2 className="font-mono text-2xl font-semibold text-slate-950" id={headingId(builtin.name)}>
        {builtin.name}
      </h2>
      <p className="mt-3 font-mono text-sm text-slate-500">{builtin.signature}</p>
      <p className="mt-7 text-lg leading-8 text-slate-600">{builtin.description}</p>

      <CodeSnippet className="mt-7">{example}</CodeSnippet>
      <p className="mt-5 text-base leading-7 text-slate-600">{exampleNote}</p>
    </section>
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

const docsLayoutRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs",
  component: DocsLayout,
});

const docsRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "/",
  component: () => <DocsContentPage page={docsPages[0]} />,
});

const sourceModesRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "source-modes",
  component: () => <DocsContentPage page={docsPages[1]} />,
});

const phpBuiltinsRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins",
  component: PhpBuiltinsPage,
});

const phpBuiltinStringsRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/strings",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("strings")!} />,
});

const phpBuiltinArraysRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/arrays",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("arrays")!} />,
});

const phpBuiltinTypesRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/types",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("types")!} />,
});

const phpBuiltinMathRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/math",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("math")!} />,
});

const phpBuiltinFilesystemRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/filesystem",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("filesystem")!} />,
});

const phpBuiltinReflectionRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/reflection",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("reflection")!} />,
});

const phpBuiltinShellRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/shell",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("shell")!} />,
});

const phpBuiltinOutputBufferingRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/output-buffering",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("output-buffering")!} />,
});

const phpBuiltinCoreRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/core",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("core")!} />,
});

const sourceBuildsRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "source-builds",
  component: () => <DocsContentPage page={docsPages[2]} />,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  docsLayoutRoute.addChildren([
    docsRoute,
    sourceModesRoute,
    phpBuiltinsRoute,
    phpBuiltinStringsRoute,
    phpBuiltinArraysRoute,
    phpBuiltinTypesRoute,
    phpBuiltinMathRoute,
    phpBuiltinFilesystemRoute,
    phpBuiltinReflectionRoute,
    phpBuiltinShellRoute,
    phpBuiltinOutputBufferingRoute,
    phpBuiltinCoreRoute,
    sourceBuildsRoute,
  ]),
]);

export const router = createRouter({
  defaultViewTransition: {
    types: ({ pathChanged }) => {
      if (!pathChanged) {
        return false;
      }

      return ["route-transition"];
    },
  },
  routeTree,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
