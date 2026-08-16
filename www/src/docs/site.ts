/**
 * Public site chrome: homepage, primary nav, Documents hub catalog, and the
 * discovery files (`/sitemap.xml`, `/robots.txt`). Pages and tests load this
 * module; do not duplicate the lists in UI or fixtures.
 */

/** Live public host. Do not emit github.io URLs in discovery files. */
export const publicSiteOrigin = "https://xo.run";

/** Top-level routes that are not docs pages. */
export const publicSurfacePaths = ["/", "/install", "/try"] as const;

/**
 * Legal routes stay out of the sitemap until the pages exist.
 * Do not add these paths here when inventing placeholder URLs.
 */
export const omittedCatalogPaths = ["/privacy", "/terms"] as const;

export type SiteNavItem = {
  label: string;
  to: string;
};

export type SiteLink = {
  title: string;
  to: string;
  description: string;
};

export type DocsCatalogGroup = {
  title: string;
  description: string;
  entries: SiteLink[];
};

export type HomePageContent = {
  definition: string;
  lead: string;
  sampleCaption: string;
  sample: string;
  links: SiteLink[];
};

/** Top-bar links. The logo goes home. Install stays the solid CTA. */
export const primaryNav: SiteNavItem[] = [
  { label: "Documents", to: "/docs" },
  { label: "Packages", to: "/docs/std" },
  { label: "Echo 2026", to: "/e26" },
  { label: "Try", to: "/try" },
];

export const installCta: SiteNavItem = { label: "Install", to: "/install" };

export type FooterLink = {
  label: string;
  href: string;
  disabled?: boolean;
};

export type FooterLinkGroup = {
  title: string;
  links: FooterLink[];
};

export const footerLinkGroups: FooterLinkGroup[] = [
  {
    title: "Learn",
    links: [
      { label: "Install", href: "/install" },
      { label: "Try Echo", href: "/try" },
      { label: "First program", href: "/docs/first-program" },
      { label: "Documents", href: "/docs" },
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

export const tryPage = {
  title: "Try Echo",
  lead: "This page checks with the shared compiler frontend. A playground run then executes the checked program and captures io.print. Filesystem, net, process, and tasks stay unavailable here. Install xo to compile through LLVM.",
};

/**
 * Internal homepage, nav, Documents hub, and footer destinations.
 * Each path must ship as a real HTML file, or the link must come down.
 */
export function publicChromePaths(): string[] {
  const paths = new Set<string>();

  for (const item of primaryNav) {
    paths.add(item.to);
  }
  paths.add(installCta.to);
  for (const link of homePage.links) {
    paths.add(link.to);
  }
  for (const group of docsHubCatalog) {
    for (const entry of group.entries) {
      paths.add(entry.to);
    }
  }
  for (const group of footerLinkGroups) {
    for (const link of group.links) {
      if (!link.disabled && link.href.startsWith("/")) {
        paths.add(link.href);
      }
    }
  }

  return [...paths].sort();
}

/**
 * Documents stays active on language and guide pages. Packages owns /docs/std.
 */
export function primaryNavItemIsActive(to: string, pathname: string): boolean {
  if (to === "/docs") {
    return (
      pathname === "/docs" || (pathname.startsWith("/docs/") && !pathname.startsWith("/docs/std"))
    );
  }
  if (pathname === to) {
    return true;
  }
  return to !== "/" && pathname.startsWith(`${to}/`);
}

export const homePage: HomePageContent = {
  definition: "Echo is a compiled language.",
  lead: "Statement leaders mark control and binding. The rest of each line is an ordinary expression. xo checks a program and emits a native binary from the same LLVM pipeline.",
  sampleCaption: "sum.echo",
  sample: `/ std/io

$ xs = [1, 2, 3]
~ sum = 0
* x : xs {
    ~ sum = sum + x
}
io.print("sum={sum}")`,
  links: [
    {
      title: "Documents",
      to: "/docs",
      description: "Language forms, the first program, and the toolchain.",
    },
    {
      title: "Packages",
      to: "/docs/std",
      description: "Shipped std modules and the API index.",
    },
    {
      title: "Spec",
      to: "/e26",
      description: "Echo 2026 language law and the conformance suite.",
    },
  ],
};

/** Criterion-4 language pages. The Documents hub Language group uses this list. */
export const languageFeatureEntries: SiteLink[] = [
  {
    title: "Leaders",
    to: "/docs/leaders",
    description: "Statement leader forms. A glyph marks the role of each statement.",
  },
  {
    title: "Binds and functions",
    to: "/docs/binds",
    description: "Immutable, mutable, and const binds. Functions are values.",
  },
  {
    title: "Values and operators",
    to: "/docs/values",
    description: "Literal forms, operator precedence, equality, and copy.",
  },
  {
    title: "Collections and ranges",
    to: "/docs/collections",
    description: "Lists, products, indexing, and inclusive ranges.",
  },
  {
    title: "Control",
    to: "/docs/control",
    description: "If, loops, match, break, and continue.",
  },
  {
    title: "Result and option",
    to: "/docs/result-option",
    description: "Produce and match result and option shapes.",
  },
  {
    title: "Strings",
    to: "/docs/strings",
    description: "Pure and rich string forms.",
  },
  {
    title: "Modules and std",
    to: "/docs/modules",
    description: "Imports, exports, and the std/ import policy.",
  },
  {
    title: "Structs",
    to: "/docs/structs",
    description: "% shape, @ members, and the method receiver.",
  },
  {
    title: "Tasks",
    to: "/docs/tasks",
    description: "Spawn work, capture values, and join task handles.",
  },
];

export const docsHubCatalog: DocsCatalogGroup[] = [
  {
    title: "Start",
    description: "Build xo and write a first program.",
    entries: [
      {
        title: "Install",
        to: "/install",
        description: "Install the current xo prerelease, or build from a checkout.",
      },
      {
        title: "First program",
        to: "/docs/first-program",
        description: "Minimal runnable shape and the xo check, run, and build commands.",
      },
      {
        title: "Project setup",
        to: "/docs/project",
        description: "Create an entry file and a local workflow.",
      },
    ],
  },
  {
    title: "Language",
    description: "Statement forms for the Echo 2026 edition.",
    entries: languageFeatureEntries,
  },
  {
    title: "Packages",
    description: "Userland modules imported as std/ paths.",
    entries: [
      {
        title: "Standard library",
        to: "/docs/std",
        description:
          "Userland modules for I/O, strings, files, process, JSON, encoding, crypto, collections, and networking.",
      },
      {
        title: "API reference",
        to: "/docs/std/reference",
        description: "Index of public exports across shipped std modules.",
      },
    ],
  },
  {
    title: "Spec",
    description: "Public language law and the machine-checked suite.",
    entries: [
      {
        title: "Echo 2026",
        to: "/e26",
        description: "Edition home for the current public language.",
      },
      {
        title: "Language Spec",
        to: "/e26/spec",
        description: "Spec table of contents mapped to the Reference and the suite.",
      },
    ],
  },
];

/**
 * Public catalog paths for `/sitemap.xml`.
 * `existingPagePaths` is the shipped HTML catalog (`staticPages` / `docsPages`).
 * Privacy and Terms are included only when those pages already exist.
 */
export function collectPublicCatalogPaths(existingPagePaths: readonly string[]): string[] {
  const existing = new Set(existingPagePaths);
  const paths = new Set<string>([
    ...publicSurfacePaths,
    ...publicChromePaths(),
    ...existingPagePaths,
  ]);

  for (const omitted of omittedCatalogPaths) {
    if (!existing.has(omitted)) {
      paths.delete(omitted);
    }
  }

  return [...paths].sort((left, right) => left.localeCompare(right));
}

export function publicCatalogUrl(path: string): string {
  if (!path.startsWith("/")) {
    throw new Error(`catalog path must be absolute, got ${path}`);
  }
  if (path === "/") {
    return `${publicSiteOrigin}/`;
  }
  return `${publicSiteOrigin}${path}`;
}

export function renderSitemapXml(paths: readonly string[]): string {
  const urls = paths
    .map((path) => `  <url>\n    <loc>${escapeHtml(publicCatalogUrl(path))}</loc>\n  </url>`)
    .join("\n");

  return [
    `<?xml version="1.0" encoding="UTF-8"?>`,
    `<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">`,
    urls,
    `</urlset>`,
    ``,
  ].join("\n");
}

export function renderRobotsTxt(): string {
  return [
    "User-agent: *",
    "Allow: /",
    "",
    `Sitemap: ${publicSiteOrigin}/sitemap.xml`,
    "",
  ].join("\n");
}

function escapeHtml(text: string): string {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function renderLinkList(links: readonly SiteLink[]): string {
  return links
    .map(
      (link) =>
        `<li><a href="${escapeHtml(link.to)}">${escapeHtml(link.title)}</a> ${escapeHtml(link.description)}</li>`,
    )
    .join("");
}

/**
 * No-JS snapshot of the homepage and Documents hub, built from the same
 * objects the React pages render. The build injects this into index.html.
 */
export function renderStaticHomeAndHub(): string {
  const surfaceLinks = renderLinkList(homePage.links);
  const catalog = docsHubCatalog
    .map((group) => {
      return [
        `<section>`,
        `<h2>${escapeHtml(group.title)}</h2>`,
        `<p>${escapeHtml(group.description)}</p>`,
        `<ul>${renderLinkList(group.entries)}</ul>`,
        `</section>`,
      ].join("");
    })
    .join("");

  return [
    `<main>`,
    `<h1>${escapeHtml(homePage.definition)}</h1>`,
    `<p>${escapeHtml(homePage.lead)}</p>`,
    `<pre>${escapeHtml(homePage.sample)}</pre>`,
    `<nav aria-label="Language surfaces"><ul>${surfaceLinks}</ul></nav>`,
    catalog,
    `</main>`,
  ].join("");
}
