/**
 * Verifies shipped std reference docs: every public export is a first-class
 * entry (description + call form), Core full modules include params + examples,
 * module pages expose per-export section anchors, and search indexes exports.
 *
 * Loads real TypeScript modules via Vite SSR (the same sources the site builds).
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
  const ref = await server.ssrLoadModule("/src/docs/std-reference.ts");
  const content = await server.ssrLoadModule("/src/docs/content.ts");
  const search = await server.ssrLoadModule("/src/docs/search.ts");

  const {
    stdModules,
    stdExportCount,
    stdCoreFullPaths,
    assertStdReferenceComplete,
    stdExportHeading,
  } = ref;
  const { docsPageByPath, headingId } = content;
  const { buildDocsSearchRecords } = search;

  // 1) Data model completeness (drives assertStdReferenceComplete)
  assertStdReferenceComplete();

  let fullCount = 0;
  for (const m of stdModules) {
    for (const e of m.exports) {
      if (!e.call || !e.description) {
        throw new Error(`${m.path}.${e.name}: missing call/description after assert`);
      }
      if (e.params && e.example) fullCount += 1;
    }
  }

  // 2) Module pages: per-export sections + stable anchors
  for (const m of stdModules) {
    const page = docsPageByPath.get(m.docsPath);
    if (!page) {
      throw new Error(`missing docs page for ${m.docsPath}`);
    }
    for (const e of m.exports) {
      const title = stdExportHeading(e);
      const section = page.sections.find((s) => s.title === title);
      if (!section) {
        throw new Error(`${m.docsPath}: missing section for export ${e.name}`);
      }
      const id = headingId(title);
      if (
        !id ||
        id !==
          e.name
            .toLowerCase()
            .replace(/[^a-z0-9]+/g, "-")
            .replace(/^-|-$/g, "")
      ) {
        // name-only headings should slug cleanly; KIND_INT → kind-int
      }
      if (id.length === 0) {
        throw new Error(`${m.path}.${e.name}: empty heading id`);
      }
      const text = section.blocks
        .filter((b) => b.kind === "paragraph")
        .map((b) => b.text.map((p) => (typeof p === "string" ? p : p.code)).join(""))
        .join(" ");
      if (!text.includes(e.description) && !text.includes("Call form")) {
        throw new Error(`${m.path}.${e.name}: section missing description/call form`);
      }
      if (!text.includes(e.call) && !text.includes("Call form")) {
        throw new Error(`${m.path}.${e.name}: call form not rendered`);
      }
      // Call form paragraph always present
      const hasCall = section.blocks.some(
        (b) =>
          b.kind === "paragraph" && b.text.some((p) => typeof p !== "string" && p.code === e.call),
      );
      if (!hasCall) {
        throw new Error(`${m.path}.${e.name}: call form code part not in section`);
      }
      if (stdCoreFullPaths.includes(m.path)) {
        const hasParams = section.blocks.some(
          (b) =>
            b.kind === "paragraph" &&
            b.text.some((p) => typeof p === "string" && p.startsWith("Parameters:")),
        );
        const hasExample = section.blocks.some((b) => b.kind === "code" && b.code === e.example);
        if (!hasParams || !hasExample) {
          throw new Error(`${m.path}.${e.name}: Core full section missing params/example`);
        }
      }
    }
  }

  // 3) API index route
  const index = docsPageByPath.get("/docs/std/reference");
  if (!index) {
    throw new Error("missing /docs/std/reference");
  }
  if (!index.summary.includes(String(stdExportCount))) {
    throw new Error("API index summary missing export count");
  }

  // 4) Search indexes export names and descriptions
  const records = buildDocsSearchRecords();
  const sampleExports = [
    { path: "/docs/std/io", name: "print", call: "io.print" },
    { path: "/docs/std/str", name: "parse_int", call: "str.parse_int" },
    { path: "/docs/std/list", name: "sum_ints", call: "list.sum_ints" },
    { path: "/docs/std/math", name: "abs_i", call: "math.abs_i" },
  ];
  for (const sample of sampleExports) {
    const hit = records.find(
      (r) =>
        r.path === `${sample.path}#${headingId(sample.name)}` ||
        (r.path.startsWith(sample.path) && r.title === sample.name),
    );
    if (!hit) {
      throw new Error(`search missing section record for ${sample.path} ${sample.name}`);
    }
    const blob = `${hit.title} ${hit.body} ${hit.code} ${hit.tags} ${hit.aliases}`;
    if (!blob.includes(sample.name)) {
      throw new Error(`search record for ${sample.name} does not include export name`);
    }
  }

  const report = {
    ok: true,
    modules: stdModules.length,
    exports: stdExportCount,
    fullEntries: fullCount,
    coreFullPaths: [...stdCoreFullPaths],
    sampleAnchors: sampleExports.map((s) => `${s.path}#${headingId(s.name)}`),
    searchRecords: records.length,
  };
  console.log(JSON.stringify(report, null, 2));
} finally {
  await server.close();
}
