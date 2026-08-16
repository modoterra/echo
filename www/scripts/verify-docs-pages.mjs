/**
 * Verifies the shipped docs-first site model:
 * - homepage definition, leader-bearing sample, docs/std/spec links
 * - primary nav labels and paths
 * - Documents hub catalog groups
 * - every language-feature catalog entry is a real page with a summary
 *   and at least one Echo code block
 *
 * Loads src/docs/site.ts and src/docs/content.ts through Vite SSR.
 */
import { createServer } from "vite";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const server = await createServer({
  root,
  logLevel: "error",
  server: { middlewareMode: true },
  appType: "custom",
});

try {
  const site = await server.ssrLoadModule("/src/docs/site.ts");
  const content = await server.ssrLoadModule("/src/docs/content.ts");

  const {
    docsHubCatalog,
    homePage,
    installCta,
    languageFeatureEntries,
    primaryNav,
    primaryNavItemIsActive,
    renderStaticHomeAndHub,
  } = site;
  const { docsPageByPath } = content;

  const failures = [];

  function fail(message) {
    failures.push(message);
  }

  if (!/compiled language/i.test(homePage.definition)) {
    fail("homePage.definition must name Echo as a compiled language");
  }
  if (!/^\$ /m.test(homePage.sample) || !/^~ /m.test(homePage.sample)) {
    fail("homePage.sample must show $ and ~ statement leaders");
  }

  const homeTargets = new Set(homePage.links.map((link) => link.to));
  for (const required of ["/docs", "/docs/std", "/e26"]) {
    if (!homeTargets.has(required)) {
      fail(`homePage.links missing ${required}`);
    }
  }

  const navByLabel = new Map(primaryNav.map((item) => [item.label, item.to]));
  const expectedNav = [
    ["Documents", "/docs"],
    ["Packages", "/docs/std"],
    ["Echo 2026", "/e26"],
    ["Try", "/try"],
  ];
  for (const [label, to] of expectedNav) {
    if (navByLabel.get(label) !== to) {
      fail(`primaryNav ${label} should be ${to}, got ${navByLabel.get(label)}`);
    }
  }
  if (installCta.label !== "Install" || installCta.to !== "/install") {
    fail("installCta must be Install → /install");
  }

  if (!primaryNavItemIsActive("/docs", "/docs/leaders")) {
    fail("Documents nav should be active on /docs/leaders");
  }
  if (primaryNavItemIsActive("/docs", "/docs/std/io")) {
    fail("Documents nav should stay inactive on package pages");
  }
  if (!primaryNavItemIsActive("/docs/std", "/docs/std/io")) {
    fail("Packages nav should be active on /docs/std/io");
  }
  if (!primaryNavItemIsActive("/e26", "/e26/spec")) {
    fail("Echo 2026 nav should be active on /e26/spec");
  }

  const groups = new Map(docsHubCatalog.map((group) => [group.title, group]));
  for (const title of ["Start", "Language", "Packages", "Spec"]) {
    if (!groups.has(title)) {
      fail(`docsHubCatalog missing group ${title}`);
    }
  }

  const startPaths = new Set(groups.get("Start")?.entries.map((entry) => entry.to) ?? []);
  if (!startPaths.has("/install") || !startPaths.has("/docs/first-program")) {
    fail("Start catalog must link to /install and /docs/first-program");
  }

  const languageGroup = groups.get("Language");
  if (languageGroup && languageGroup.entries !== languageFeatureEntries) {
    fail("Language catalog entries must be the languageFeatureEntries array");
  }

  const packagePaths = new Set(groups.get("Packages")?.entries.map((entry) => entry.to) ?? []);
  if (!packagePaths.has("/docs/std")) {
    fail("Packages catalog must link to /docs/std");
  }

  const specPaths = new Set(groups.get("Spec")?.entries.map((entry) => entry.to) ?? []);
  if (!specPaths.has("/e26") && !specPaths.has("/e26/spec")) {
    fail("Spec catalog must link to /e26 or /e26/spec");
  }

  for (const entry of languageFeatureEntries) {
    if (!entry.title.trim() || !entry.description.trim()) {
      fail(`${entry.to}: catalog entry needs a title and description`);
      continue;
    }
    const page = docsPageByPath.get(entry.to);
    if (!page) {
      fail(`${entry.to}: missing from docsPageByPath`);
      continue;
    }
    if (!page.summary.trim()) {
      fail(`${entry.to}: empty summary`);
    }
    const hasEchoCode = page.sections.some((section) =>
      section.blocks.some(
        (block) => block.kind === "code" && (block.language ?? "echo") === "echo",
      ),
    );
    if (!hasEchoCode) {
      fail(`${entry.to}: needs at least one Echo code block`);
    }
  }

  const snapshot = renderStaticHomeAndHub();
  for (const needle of [
    homePage.definition,
    homePage.sample.trim().split("\n")[0],
    "/docs",
    "/docs/std",
    "/e26",
    "/docs/leaders",
    "/install",
  ]) {
    if (!snapshot.includes(needle)) {
      fail(`renderStaticHomeAndHub missing ${needle}`);
    }
  }

  if (failures.length) {
    console.error(JSON.stringify({ ok: false, failures }, null, 2));
    process.exitCode = 1;
  } else {
    console.log(
      JSON.stringify(
        {
          ok: true,
          definition: homePage.definition,
          nav: primaryNav.map((item) => item.label),
          languagePages: languageFeatureEntries.map((entry) => entry.to),
          hubGroups: docsHubCatalog.map((group) => group.title),
        },
        null,
        2,
      ),
    );
  }
} finally {
  await server.close();
}
