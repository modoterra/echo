import {
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Outlet,
  useNavigate,
  useLocation,
} from "@tanstack/react-router";
import {
  RiArrowRightLine,
  RiBookOpenLine,
  RiBracketsLine,
  RiCheckLine,
  RiCloseLine,
  RiCodeBoxLine,
  RiCodeLine,
  RiFileCopyLine,
  RiFileSearchLine,
  RiFileTextLine,
  RiFingerprintLine,
  RiFolderOpenLine,
  RiFunctionLine,
  RiNumbersLine,
  RiPhpLine,
  RiRocketLine,
  RiSearchLine,
  RiSearchEyeLine,
  RiSettingsLine,
  RiShieldUserLine,
  RiStackLine,
  RiTerminalBoxLine,
  type RemixiconComponentType,
} from "@remixicon/react";
import { layout, prepare } from "@chenglou/pretext";
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
import { createPortal } from "react-dom";
import { docsSearchIndexUrl, docsSemanticIndexUrl } from "virtual:docs-search-indices";
import { HomePage } from "./app";
import {
  builtinExample,
  builtinExampleNote,
  builtinFamilies,
  builtinFamilyBySlug,
  docsNavigation,
  docsPageByPath,
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

function docsPage(path: string) {
  const page = docsPageByPath.get(path);
  if (!page) {
    throw new Error(`Missing docs page: ${path}`);
  }
  return page;
}

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

type ShikiCoreModule = typeof import("react-shiki/core");
type PhpHighlighter = Awaited<ReturnType<ShikiCoreModule["createHighlighterCore"]>>;
type ShikiHighlighterComponent = ShikiCoreModule["default"];

let phpHighlighterPromise: Promise<{
  highlighter: PhpHighlighter;
  ShikiHighlighter: ShikiHighlighterComponent;
}> | null = null;

const codeSnippetFont = '14px "Geist Mono"';
const codeSnippetLineHeight = 28;
const codeSnippetBlockPadding = 48;
const codeSnippetSkeletonMinDelay = 120;
const codeSnippetSkeletonMaxDelay = 280;
const codeSnippetLoadRootMargin = "0px";

function loadPhpHighlighter() {
  phpHighlighterPromise ??= Promise.all([
    import("react-shiki/core"),
    import("@shikijs/langs/php"),
    import("@shikijs/themes/github-dark"),
  ]).then(async ([shikiCore, php, githubDark]) => {
    const highlighter = await shikiCore.createHighlighterCore({
      engine: shikiCore.createJavaScriptRegexEngine({ forgiving: true }),
      langs: [php.default],
      themes: [githubDark.default],
    });

    return {
      highlighter,
      ShikiHighlighter: shikiCore.default,
    };
  });

  return phpHighlighterPromise;
}

function randomCodeSnippetSkeletonDelay() {
  const range = codeSnippetSkeletonMaxDelay - codeSnippetSkeletonMinDelay;

  return codeSnippetSkeletonMinDelay + Math.round(Math.random() * range);
}

type DocsSearchResult = Pick<
  DocsSearchRecord,
  "id" | "path" | "title" | "category" | "kind" | "excerpt" | "signature"
> & {
  score: number;
  lexicalScore?: number;
  semanticScore?: number;
};

const docsSearchResultLimit = 8;
const docsSearchLexicalCandidateLimit = 24;
const docsSearchSemanticCandidateLimit = 24;
const docsSearchLexicalWeight = 0.6;
const docsSearchSemanticWeight = 0.4;
const instantSearchHashScroll = {
  behavior: "auto",
  block: "start",
} as const satisfies ScrollIntoViewOptions;

let docsSearchAssetPromise: Promise<DocsSearchAsset> | null = null;
let docsSemanticAssetPromise: Promise<DocsSemanticAsset> | null = null;

function mergeHybridSearchResults({
  lexicalResults,
  recordById,
  semanticResults,
}: {
  lexicalResults: DocsSearchResult[];
  recordById: Map<string, DocsSearchRecord>;
  semanticResults: { id: string; score: number }[];
}) {
  const maxLexicalScore = Math.max(1, ...lexicalResults.map((result) => result.score));
  const maxSemanticScore = Math.max(0.0001, ...semanticResults.map((result) => result.score));
  const merged = new Map<string, DocsSearchResult>();

  for (const result of lexicalResults) {
    merged.set(result.id, {
      ...result,
      lexicalScore: result.score / maxLexicalScore,
      score: (result.score / maxLexicalScore) * docsSearchLexicalWeight,
    });
  }

  for (const semanticResult of semanticResults) {
    const normalizedSemanticScore = semanticResult.score / maxSemanticScore;
    const existing = merged.get(semanticResult.id);

    if (existing) {
      existing.semanticScore = normalizedSemanticScore;
      existing.score += normalizedSemanticScore * docsSearchSemanticWeight;
      continue;
    }

    const record = recordById.get(semanticResult.id);

    if (!record) {
      continue;
    }

    merged.set(record.id, {
      id: record.id,
      path: record.path,
      title: record.title,
      category: record.category,
      kind: record.kind,
      excerpt: record.excerpt,
      score: normalizedSemanticScore * docsSearchSemanticWeight,
      semanticScore: normalizedSemanticScore,
    });
  }

  return Array.from(merged.values())
    .sort((left, right) => right.score - left.score)
    .slice(0, docsSearchResultLimit);
}

function DocsSearch() {
  const navigate = useNavigate();
  const [isOpen, setIsOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [activeResultIndex, setActiveResultIndex] = useState(0);
  const [asset, setAsset] = useState<DocsSearchAsset | null>(null);
  const [semanticAsset, setSemanticAsset] = useState<DocsSemanticAsset | null>(null);
  const [queryEmbedding, setQueryEmbedding] = useState<number[] | null>(null);
  const [isLoadingIndex, setIsLoadingIndex] = useState(false);
  const [isSemanticModelReady, setIsSemanticModelReady] = useState(false);
  const [semanticUnavailable, setSemanticUnavailable] = useState(false);
  const searchInputRef = useRef<HTMLInputElement | null>(null);
  const searchResultsRef = useRef<HTMLDivElement | null>(null);
  const searchResultRefs = useRef<Record<string, HTMLLIElement | null>>({});
  const miniSearch = useMemo(() => (asset ? loadDocsMiniSearch(asset) : null), [asset]);
  const results = useMemo(() => {
    const trimmedQuery = query.trim();

    if (!miniSearch || !trimmedQuery) {
      return [];
    }

    const lexicalResults = miniSearch.search(trimmedQuery) as unknown as DocsSearchResult[];

    if (!asset || !semanticAsset || !queryEmbedding) {
      return lexicalResults.slice(0, docsSearchResultLimit);
    }

    const recordById = new Map(asset.records.map((record) => [record.id, record]));
    const semanticResults = semanticAsset.records
      .map((record) => ({
        id: record.id,
        score: cosineSimilarity(queryEmbedding, record.embedding),
      }))
      .sort((left, right) => right.score - left.score)
      .slice(0, docsSearchSemanticCandidateLimit);

    return mergeHybridSearchResults({
      lexicalResults: lexicalResults.slice(0, docsSearchLexicalCandidateLimit),
      recordById,
      semanticResults,
    });
  }, [asset, miniSearch, query, queryEmbedding, semanticAsset]);
  const activeResult = results[activeResultIndex];

  useEffect(() => {
    if (!isOpen || asset) {
      return;
    }

    let active = true;
    setIsLoadingIndex(true);

    void loadDocsSearchAsset()
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
    setActiveResultIndex(0);
  }, [query]);

  useLayoutEffect(() => {
    if (!isOpen || !activeResult) {
      return;
    }

    const container = searchResultsRef.current;
    const item = searchResultRefs.current[activeResult.id];

    if (!container || !item) {
      return;
    }

    scrollElementIntoContainerView(container, item, "smooth");
  }, [activeResult, isOpen]);

  useEffect(() => {
    if (!isOpen || semanticAsset || semanticUnavailable) {
      return;
    }

    let active = true;

    void loadDocsSemanticAsset()
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
  }, [isOpen, semanticAsset, semanticUnavailable]);

  useEffect(() => {
    if (!isOpen || !semanticAsset || isSemanticModelReady || semanticUnavailable) {
      return;
    }

    let active = true;

    void preloadSearchEmbedder()
      .then(() => {
        if (active) {
          setIsSemanticModelReady(true);
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
  }, [isOpen, isSemanticModelReady, semanticAsset, semanticUnavailable]);

  useEffect(() => {
    const trimmedQuery = query.trim();

    if (!semanticAsset || !trimmedQuery) {
      setQueryEmbedding(null);
      return;
    }

    let active = true;

    void embedSearchQuery(trimmedQuery).then((embedding) => {
      if (active) {
        setQueryEmbedding(embedding);
      }
    });

    return () => {
      active = false;
    };
  }, [query, semanticAsset]);

  useEffect(() => {
    function handleGlobalSearchKey(event: KeyboardEvent) {
      const target = event.target;
      const isEditableTarget =
        target instanceof HTMLElement &&
        (target.isContentEditable || target.matches("input, textarea, select"));

      if (isEditableTarget) {
        return;
      }

      if (
        event.key === "/" ||
        ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k")
      ) {
        event.preventDefault();
        setIsOpen(true);
      }
    }

    window.addEventListener("keydown", handleGlobalSearchKey);

    return () => {
      window.removeEventListener("keydown", handleGlobalSearchKey);
    };
  }, []);

  useEffect(() => {
    function handlePaletteKey(event: KeyboardEvent) {
      if (event.key === "Escape") {
        closeSearch();
        return;
      }

      if (event.key === "ArrowDown") {
        event.preventDefault();
        setActiveResultIndex((index) => Math.min(index + 1, results.length - 1));
        return;
      }

      if (event.key === "ArrowUp") {
        event.preventDefault();
        setActiveResultIndex((index) => Math.max(index - 1, 0));
        return;
      }

      if (event.key === "Enter" && activeResult) {
        void navigate({
          hashScrollIntoView: instantSearchHashScroll,
          to: activeResult.path,
          viewTransition: false,
        });
        closeSearch();
      }
    }

    if (isOpen) {
      window.addEventListener("keydown", handlePaletteKey);
    }

    return () => {
      window.removeEventListener("keydown", handlePaletteKey);
    };
  }, [activeResult, isOpen, navigate, results.length]);

  function closeSearch() {
    setIsOpen(false);
    setQuery("");
    setActiveResultIndex(0);
    setQueryEmbedding(null);
    searchResultsRef.current?.scrollTo({ top: 0 });
  }

  const isSemanticReady = Boolean(semanticAsset) && isSemanticModelReady;

  return (
    <>
      <button
        aria-expanded={isOpen}
        aria-label="Search documentation"
        className="inline-flex h-9 items-center gap-2 rounded-md border border-slate-200 bg-white px-3 text-sm font-semibold text-slate-500 transition hover:border-slate-300 hover:text-slate-950"
        onClick={() => setIsOpen(true)}
        type="button"
      >
        <RiSearchLine size={16} />
        <span className="hidden sm:inline">Search</span>
        <span className="hidden rounded border border-slate-200 px-1.5 py-0.5 text-xs text-slate-400 lg:inline">
          /
        </span>
      </button>
      {createPortal(
        <AnimatePresence>
          {isOpen ? (
            <motion.div
              animate={{ opacity: 1 }}
              className="fixed inset-0 z-50 flex items-start justify-center bg-slate-950/05 px-4 pt-28 backdrop-blur-xs"
              exit={{ opacity: 0 }}
              initial={{ opacity: 0 }}
              onMouseDown={closeSearch}
              transition={{ duration: 0.16, ease: "easeOut" }}
            >
              <motion.div
                animate={{ opacity: 1, scale: 1, y: 0 }}
                className="w-full max-w-2xl overflow-hidden rounded-lg border border-slate-200 bg-white shadow-2xl"
                exit={{ opacity: 0, scale: 0.98, y: -6 }}
                initial={{ opacity: 0, scale: 0.98, y: -6 }}
                onMouseDown={(event) => event.stopPropagation()}
                role="dialog"
                aria-modal="true"
                aria-label="Search documentation"
                transition={{ duration: 0.18, ease: "easeOut" }}
              >
                <div className="flex h-16 items-center gap-3 border-b border-slate-200 px-5">
                  <RiSearchLine className="shrink-0 text-slate-400" size={22} />
                  <input
                    className="h-full min-w-0 flex-1 bg-transparent text-lg text-slate-950 outline-none placeholder:text-slate-400"
                    onChange={(event) => setQuery(event.target.value)}
                    placeholder="Search docs"
                    ref={searchInputRef}
                    value={query}
                  />
                  {query ? (
                    <button
                      aria-label="Clear search"
                      className="inline-flex size-8 items-center justify-center rounded-md text-slate-400 transition hover:bg-slate-100 hover:text-slate-950"
                      onClick={() => setQuery("")}
                      type="button"
                    >
                      <RiCloseLine size={20} />
                    </button>
                  ) : null}
                  <button
                    aria-label="Close search"
                    className="hidden rounded-md border border-slate-200 bg-white px-2 py-1 text-xs font-semibold text-slate-400 transition hover:border-slate-300 hover:bg-slate-50 hover:text-slate-950 sm:inline"
                    onClick={closeSearch}
                    type="button"
                  >
                    Esc
                  </button>
                </div>
                <div className="flex min-h-10 items-center justify-between border-b border-slate-100 px-5 py-2 text-xs font-semibold text-slate-400">
                  <div className="flex items-center gap-2">
                    <span
                      className={
                        isSemanticReady
                          ? "inline-flex items-center gap-2 rounded-md border border-sky-200 bg-sky-50 px-2.5 py-1 text-sky-700"
                          : "inline-flex items-center gap-2 rounded-md border border-slate-200 bg-slate-50 px-2.5 py-1 text-slate-400"
                      }
                    >
                      <span
                        className={
                          isSemanticReady
                            ? "size-1.5 rounded-full bg-sky-500"
                            : "size-1.5 rounded-full bg-slate-300"
                        }
                      />
                      Semantic
                    </span>
                  </div>
                  <span className="hidden text-slate-300 sm:inline">
                    <span className="font-mono">↑↓</span> Select{" "}
                    <span className="font-mono">Enter</span> Open
                  </span>
                </div>
                <div
                  className="max-h-[28rem] overflow-auto p-3 scrollbar-thin scrollbar-nice"
                  ref={searchResultsRef}
                >
                  {!query.trim() ? (
                    <p className="px-3 py-10 text-center text-sm text-slate-500">
                      Search built-ins, examples, commands, and docs.
                    </p>
                  ) : null}
                  {query.trim() && !isLoadingIndex && results.length === 0 ? (
                    <p className="px-3 py-10 text-center text-sm text-slate-500">
                      No results found.
                    </p>
                  ) : null}
                  <ul className="space-y-1">
                    {results.map((result, index) => {
                      const isActive = index === activeResultIndex;

                      return (
                        <li
                          key={result.id}
                          ref={(element) => {
                            searchResultRefs.current[result.id] = element;
                          }}
                        >
                          <Link
                            className={
                              isActive
                                ? "grid grid-cols-[2rem_minmax(0,1fr)_1.25rem] gap-3 rounded-md bg-slate-100 px-3 py-3 text-slate-950"
                                : "grid grid-cols-[2rem_minmax(0,1fr)_1.25rem] gap-3 rounded-md px-3 py-3 text-slate-600 transition hover:bg-slate-50 hover:text-slate-950"
                            }
                            onClick={closeSearch}
                            onMouseEnter={() => setActiveResultIndex(index)}
                            hashScrollIntoView={instantSearchHashScroll}
                            to={result.path}
                            viewTransition={false}
                          >
                            <span className="mt-1 text-slate-400">
                              <SearchResultIcon result={result} />
                            </span>
                            <span className="min-w-0">
                              <span className="block truncate text-sm font-semibold">
                                {result.title}
                              </span>
                              <span className="mt-1 block text-xs font-semibold text-slate-400">
                                {searchResultMeta(result)}
                              </span>
                              {result.signature ? (
                                <span className="mt-2 block truncate font-mono text-sm text-slate-600">
                                  {result.signature}
                                </span>
                              ) : null}
                              <span className="mt-1 line-clamp-2 block text-sm leading-6 text-slate-500">
                                {result.excerpt}
                              </span>
                            </span>
                            <span className="flex size-5 items-center justify-center self-center text-slate-400">
                              {isActive ? <RiArrowRightLine size={18} /> : null}
                            </span>
                          </Link>
                        </li>
                      );
                    })}
                  </ul>
                </div>
              </motion.div>
            </motion.div>
          ) : null}
        </AnimatePresence>,
        document.body,
      )}
    </>
  );
}

function SearchResultIcon({ result }: { result: DocsSearchResult }) {
  const Icon = searchResultIcon(result);

  return <Icon size={20} />;
}

function searchResultMeta(result: DocsSearchResult) {
  return `${result.category} ${result.kind}`;
}

function searchResultIcon(result: DocsSearchResult): RemixiconComponentType {
  if (result.kind === "code") {
    return RiCodeBoxLine;
  }

  if (result.kind === "function") {
    return RiFunctionLine;
  }

  if (result.kind === "builtin") {
    return builtinCategoryIcon(result.category);
  }

  switch (result.category) {
    case "Getting Started":
      return RiRocketLine;
    case "Tooling":
      return RiTerminalBoxLine;
    case "Language":
      return RiBookOpenLine;
    case "PHP Built-ins":
      return RiPhpLine;
    default:
      return result.kind === "section" ? RiFileSearchLine : RiFileTextLine;
  }
}

function builtinCategoryIcon(category: string): RemixiconComponentType {
  switch (category) {
    case "Strings":
      return RiCodeLine;
    case "Arrays":
      return RiBracketsLine;
    case "Types":
      return RiShieldUserLine;
    case "Math":
      return RiNumbersLine;
    case "Hashes":
      return RiFingerprintLine;
    case "Filesystem":
      return RiFolderOpenLine;
    case "Reflection":
      return RiSearchEyeLine;
    case "Shell":
      return RiTerminalBoxLine;
    case "Output Buffering":
      return RiStackLine;
    case "Core":
      return RiSettingsLine;
    default:
      return RiFunctionLine;
  }
}

function loadDocsSearchAsset() {
  docsSearchAssetPromise ??= fetchDocsIndex<DocsSearchAsset>(docsSearchIndexUrl);

  return docsSearchAssetPromise;
}

function loadDocsSemanticAsset() {
  docsSemanticAssetPromise ??= fetchDocsIndex<DocsSemanticAsset>(docsSemanticIndexUrl);

  return docsSemanticAssetPromise;
}

async function fetchDocsIndex<T>(path: string) {
  const response = await fetch(path, { cache: "force-cache" });
  const contentType = response.headers.get("Content-Type") ?? "";

  if (!response.ok || !contentType.includes("application/json")) {
    throw new Error(`Docs index is not available: ${path}`);
  }

  return (await response.json()) as T;
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

let queryEmbedderPromise: Promise<{
  (
    query: string,
    options: { pooling: "mean"; normalize: true },
  ): Promise<{
    data: ArrayLike<number>;
  }>;
}> | null = null;

function preloadSearchEmbedder() {
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

  return queryEmbedderPromise;
}

async function embedSearchQuery(query: string) {
  const embedder = await preloadSearchEmbedder();
  const output = await embedder(query, { pooling: "mean", normalize: true });

  return Array.from(output.data);
}

function CodeSnippet({ children, className = "mt-8" }: { children: string; className?: string }) {
  const snippetRef = useRef<HTMLDivElement | null>(null);
  const [copied, setCopied] = useState(false);
  const [shiki, setShiki] = useState<{
    highlighter: PhpHighlighter;
    ShikiHighlighter: ShikiHighlighterComponent;
  } | null>(null);
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

    void Promise.all([loadPhpHighlighter(), delay]).then(([loadedShiki]) => {
      if (active) {
        setShiki(loadedShiki);
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

  const ShikiHighlighter = shiki?.ShikiHighlighter;

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
        {shiki && ShikiHighlighter ? (
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
              highlighter={shiki.highlighter}
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
          The requested Echo docs page does not exist or has moved.
        </p>
        <div className="mt-9 flex flex-wrap gap-3">
          <Link
            className="inline-flex items-center justify-center rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white transition hover:bg-slate-800"
            to="/docs"
          >
            Open docs
          </Link>
          <Link
            className="inline-flex items-center justify-center rounded-md border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-700 transition hover:border-slate-400 hover:text-slate-950"
            to="/"
          >
            Go home
          </Link>
        </div>
      </section>
    </main>
  );
}

type FooterLink =
  | { label: string; href: string; disabled?: false }
  | { label: string; disabled: true };

type FooterLinkGroup = {
  title: string;
  links: FooterLink[];
};

const footerLinkGroups: FooterLinkGroup[] = [
  {
    title: "Links",
    links: [
      { label: "Home", href: "/" },
      { label: "Getting Started", href: "/docs" },
      { label: "Installation", href: "/docs" },
      { label: "Docs", href: "/docs" },
      { label: "Roadmap", disabled: true },
    ],
  },
  {
    title: "Explore",
    links: [
      { label: "Source Modes", href: "/docs/source-modes" },
      { label: "PHP Built-ins", href: "/docs/php-built-ins" },
      { label: "Compatibility", disabled: true },
      { label: "Examples", disabled: true },
      { label: "Benchmarks", disabled: true },
    ],
  },
  {
    title: "Tooling",
    links: [
      { label: "Source Builds", href: "/docs/source-builds" },
      { label: "Command Line", disabled: true },
      { label: "Language Server", disabled: true },
      { label: "Testing", disabled: true },
    ],
  },
  {
    title: "Project",
    links: [
      { label: "Modoterra", href: "https://modoterra.xyz" },
      { label: "GitHub", href: "https://github.com/modoterra/echo" },
      { label: "Issues", href: "https://github.com/modoterra/echo/issues" },
      { label: "Releases", href: "https://github.com/modoterra/echo/releases" },
    ],
  },
];

function SiteFooter() {
  return (
    <footer className="overflow-hidden border-t border-slate-200 bg-white px-6 pt-24 text-slate-600">
      <div className="mx-auto grid w-full max-w-7xl gap-14 lg:grid-cols-[minmax(0,360px)_1fr]">
        <section>
          <p className="max-w-sm text-xl font-semibold leading-8 text-slate-950">
            PHP-compatible source today, native binaries tomorrow.
          </p>
          <p className="mt-5 max-w-sm text-sm leading-6 text-slate-500">
            Echo is an early-stage Rust implementation of a PHP superset with compiler tooling and
            native execution as the direction of travel.
          </p>
          <p className="mt-10 text-sm text-slate-400">© 2026 Modoterra Corporation</p>
        </section>

        <nav aria-label="Footer navigation" className="grid gap-10 sm:grid-cols-2 lg:grid-cols-4">
          {footerLinkGroups.map((group) => (
            <section key={group.title}>
              <h2 className="text-sm font-semibold text-slate-950">{group.title}</h2>
              <ul className="mt-6 space-y-4">
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

      <div className="mx-auto mt-96 flex w-full max-w-7xl justify-end">
        <svg
          aria-hidden="true"
          className="footer-echo-mark h-auto w-[min(72vw,58rem)] shrink-0 select-none"
          viewBox="0 0 980 300"
        >
          <defs>
            <pattern
              height="10"
              id="footer-echo-pixels"
              patternTransform="translate(0 2)"
              patternUnits="userSpaceOnUse"
              width="10"
            >
              <rect fill="url(#footer-echo-fade)" height="8.5" width="8.5" x="0" y="0" />
            </pattern>
            <linearGradient
              gradientUnits="userSpaceOnUse"
              id="footer-echo-gradient"
              x1="0"
              x2="980"
              y1="0"
              y2="0"
            >
              <stop offset="0%" stopColor="#A855F7" />
              <stop offset="38%" stopColor="#6366F1" />
              <stop offset="68%" stopColor="#0EA5E9" />
              <stop offset="100%" stopColor="#06B6D4" />
            </linearGradient>
            <linearGradient
              gradientUnits="userSpaceOnUse"
              id="footer-echo-fade"
              x1="0"
              x2="0"
              y1="0"
              y2="300"
            >
              <stop offset="0" stopColor="white" stopOpacity="0.08" />
              <stop offset="0.48" stopColor="white" stopOpacity="0.72" />
              <stop offset="0.9" stopColor="white" stopOpacity="0.82" />
              <stop offset="1" stopColor="white" stopOpacity="0.46" />
            </linearGradient>
            <mask id="footer-echo-solid-mask">
              <text
                fill="url(#footer-echo-fade)"
                fontFamily="Inter, system-ui, sans-serif"
                fontSize="290"
                fontWeight="650"
                letterSpacing="-2"
                x="0"
                y="275"
              >
                Echo
              </text>
            </mask>
            <mask id="footer-echo-text-mask">
              <text
                fill="url(#footer-echo-pixels)"
                fontFamily="Inter, system-ui, sans-serif"
                fontSize="290"
                fontWeight="650"
                letterSpacing="-2"
                x="0"
                y="275"
              >
                Echo
              </text>
            </mask>
          </defs>
          <rect
            fill="url(#footer-echo-gradient)"
            height="300"
            mask="url(#footer-echo-solid-mask)"
            opacity="0.12"
            width="980"
            x="0"
            y="0"
          />
          <rect
            fill="url(#footer-echo-gradient)"
            height="300"
            mask="url(#footer-echo-text-mask)"
            opacity="0.65"
            width="980"
            x="0"
            y="0"
          />
        </svg>
      </div>
    </footer>
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
                  className="docs-primary-nav-train docs-logo-gradient-rail absolute left-0 top-[3px] h-[18px] w-[3px] rounded-full transition-transform duration-200 ease-out"
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
  const onThisPageViewportRef = useRef<HTMLDivElement | null>(null);
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
    <main className="min-h-screen bg-white px-6 pb-24 pt-32 text-slate-950">
      <div className="mx-auto grid w-full max-w-7xl grid-cols-1 gap-12 lg:grid-cols-[220px_minmax(0,720px)] xl:grid-cols-[220px_minmax(0,720px)_220px]">
        <aside className="hidden lg:block">
          <nav
            aria-label="Documentation sections"
            className="sticky top-32 max-h-[calc(100vh-12rem)] overflow-y-auto pr-2 scrollbar-thin scrollbar-nice"
          >
            <div className="space-y-10">
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
            </div>
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
              className="mt-5 max-h-[calc(100vh-12rem)] overflow-y-auto pr-2 scrollbar-thin scrollbar-nice"
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

function renderInlineCodeText(text: string) {
  return text.split(/(`[^`]+`)/g).map((part, index) => {
    if (part.startsWith("`") && part.endsWith("`")) {
      return (
        <code className="font-mono text-slate-950" key={index}>
          {part.slice(1, -1)}
        </code>
      );
    }

    return part;
  });
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

      <div className="mt-10 grid gap-6">
        {builtinFamilies.map((family) => (
          <section
            key={family.slug}
            className="scroll-mt-28 border-t border-slate-200 pt-6"
            id={headingId(family.title)}
          >
            <h2 className="text-2xl font-semibold tracking-normal text-slate-950">
              {family.title}
            </h2>
            <p className="mt-4 text-base leading-7 text-slate-600">{family.description}</p>
            <Link
              className="mt-5 inline-flex text-sm font-semibold text-slate-500 transition hover:text-slate-950"
              to={phpBuiltinFamilyPath(family.slug)}
            >
              {family.builtins.length} functions
            </Link>
          </section>
        ))}
      </div>
    </DocsShell>
  );
}

function phpBuiltinFamilyPath(slug: string) {
  switch (slug) {
    case "strings":
      return "/docs/php-built-ins/strings";
    case "arrays":
      return "/docs/php-built-ins/arrays";
    case "types":
      return "/docs/php-built-ins/types";
    case "math":
      return "/docs/php-built-ins/math";
    case "hashes":
      return "/docs/php-built-ins/hashes";
    case "filesystem":
      return "/docs/php-built-ins/filesystem";
    case "reflection":
      return "/docs/php-built-ins/reflection";
    case "shell":
      return "/docs/php-built-ins/shell";
    case "output-buffering":
      return "/docs/php-built-ins/output-buffering";
    case "core":
      return "/docs/php-built-ins/core";
    default:
      return "/docs/php-built-ins";
  }
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
      <h2
        className="scroll-mt-28 font-mono text-2xl font-semibold text-slate-950"
        id={headingId(builtin.name)}
      >
        {builtin.name}
      </h2>
      <p className="mt-3 font-mono text-sm text-slate-500">{builtin.signature}</p>
      <p className="mt-7 text-lg leading-8 text-slate-600">{builtin.description}</p>

      <CodeSnippet className="mt-7">{example}</CodeSnippet>
      <p className="mt-5 text-base leading-7 text-slate-600">{renderInlineCodeText(exampleNote)}</p>
    </section>
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

const docsLayoutRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs",
  component: DocsLayout,
});

const docsRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "/",
  component: () => <DocsContentPage page={docsPage("/docs")} />,
});

const sourceModesRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "source-modes",
  component: () => <DocsContentPage page={docsPage("/docs/source-modes")} />,
});

const dataStructuresRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "data-structures",
  component: () => <DocsContentPage page={docsPage("/docs/data-structures")} />,
});

const dataStructuresListRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "data-structures/list",
  component: () => <DocsContentPage page={docsPage("/docs/data-structures/list")} />,
});

const dataStructuresObjectRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "data-structures/object",
  component: () => <DocsContentPage page={docsPage("/docs/data-structures/object")} />,
});

const dataStructuresClassRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "data-structures/class",
  component: () => <DocsContentPage page={docsPage("/docs/data-structures/class")} />,
});

const dataStructuresArrayRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "data-structures/array",
  component: () => <DocsContentPage page={docsPage("/docs/data-structures/array")} />,
});

const dataStructuresEnumRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "data-structures/enum",
  component: () => <DocsContentPage page={docsPage("/docs/data-structures/enum")} />,
});

const standardLibraryRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "std",
  component: () => <DocsContentPage page={docsPage("/docs/std")} />,
});

const standardLibraryNetRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "std/net",
  component: () => <DocsContentPage page={docsPage("/docs/std/net")} />,
});

const standardLibraryHttpRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "std/http",
  component: () => <DocsContentPage page={docsPage("/docs/std/http")} />,
});

const standardLibraryTimeRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "std/time",
  component: () => <DocsContentPage page={docsPage("/docs/std/time")} />,
});

const standardLibraryReflectRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "std/reflect",
  component: () => <DocsContentPage page={docsPage("/docs/std/reflect")} />,
});

const standardLibraryAssertRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "std/assert",
  component: () => <DocsContentPage page={docsPage("/docs/std/assert")} />,
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

const phpBuiltinHashesRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/hashes",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("hashes")!} />,
});

const phpBuiltinHashRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/hash",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("hashes")!} />,
});

const phpBuiltinHashesAndChecksumsRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/hashes-and-checksums",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("hashes")!} />,
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
  component: () => <DocsContentPage page={docsPage("/docs/source-builds")} />,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  docsLayoutRoute.addChildren([
    docsRoute,
    sourceModesRoute,
    dataStructuresRoute,
    dataStructuresListRoute,
    dataStructuresObjectRoute,
    dataStructuresClassRoute,
    dataStructuresArrayRoute,
    dataStructuresEnumRoute,
    standardLibraryRoute,
    standardLibraryNetRoute,
    standardLibraryHttpRoute,
    standardLibraryTimeRoute,
    standardLibraryReflectRoute,
    standardLibraryAssertRoute,
    phpBuiltinsRoute,
    phpBuiltinStringsRoute,
    phpBuiltinArraysRoute,
    phpBuiltinTypesRoute,
    phpBuiltinMathRoute,
    phpBuiltinHashesRoute,
    phpBuiltinHashRoute,
    phpBuiltinHashesAndChecksumsRoute,
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
